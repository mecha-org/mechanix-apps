use std::{io::Error, time::Duration};
use anyhow::{bail, Result};
use relm4::Sender;
use tokio::{select, sync::mpsc, time};
use crate::server::provision_client::ProvisionManagerClient;

use crate::pages::link_machine_page::InputMessage as Message;

#[derive(Debug)]
enum HandleMessage {
    GenerateCodeRes { response: Result<String> },
    ProvisionCodeRes { response: Result<bool> },
}

pub struct LinkMachineHandler {
}

impl LinkMachineHandler {

    pub fn new() -> Self {
        Self { 
        }
    }

    pub async fn run(&mut self, sender: Sender<Message>) -> Result<(), Error> {
        println!("Inside run ");
 
        let (event_tx, mut event_rx) = mpsc::channel(128);

        let (g_code_message_tx, g_code_message_rx) = mpsc::channel(128);
        let mut g_code_handler = GenerateCodeHandler::new(event_tx.clone());

        let g_code_t = tokio::spawn(async move {
            g_code_handler.run(g_code_message_rx).await;
        });


        let (p_code_message_tx, p_code_message_rx) = mpsc::channel(128);
        let mut p_code_handler = ProvisionCodeHandler::new(event_tx);
        
        let p_code_t = tokio::spawn(async move {
            p_code_handler.run(p_code_message_rx).await;
        });

        loop {
            select! {
                    event = event_rx.recv() => {
                        if event.is_none() {
                            continue;
                        }

                        println!("event received {:?}", event);
                        match event.unwrap() {
                            HandleMessage::GenerateCodeRes { response } => {
                                println!("gcode res event {:?}", response);
                                match response {
                                    Ok(code) => {
                                        let _ = p_code_message_tx.send(PCodeHandlerMessage::CodeChanged { code: code.clone() }).await;
                                        let _ = sender.send(Message::CodeChanged(code.clone()));

                                    },
                                    Err(e) => {
                                        println!("error in gcode {}", e);

                                        let _ = sender.send(Message::GenerateCodeError("Error".to_owned()));

                                    }
                                }
                            }
                            HandleMessage::ProvisionCodeRes { response } => {
                                println!("pcode res event {:?}", response);
                                match response {
                                    Ok(success) => {
                                        if success {
                                            println!("PCODE SUCCESS {:?} ", success);
                                            let _ = g_code_message_tx.send(GCodeHandlerMessage::ChangeRunningStatus { status: RunningStatus::STOP }).await;
                                            let _ = p_code_message_tx.send(PCodeHandlerMessage::ChangeRunningStatus { status: RunningStatus::STOP }).await;

                                            let _ = sender.send(Message::ProvisionSuccess);

                                        }
                                    },
                                    Err(e) => {
                                        println!("error in pcode {}", e);
                                        let _ = sender.send(Message::ProvisionSuccess);

                                    }
                                }
                            }
                        };
                    }
            }
        }
        // g_code_t.await.unwrap();
        // p_code_t.await.unwrap();

    }
  
}
  

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunningStatus {
    INACTIVE,
    START,
    STOP,
}

pub enum GCodeHandlerMessage {
    ChangeRunningStatus { status: RunningStatus },
}
struct GenerateCodeHandler {
    is_calling: bool,
    status: RunningStatus,
    event_tx: mpsc::Sender<HandleMessage>,
}

impl GenerateCodeHandler {
    pub fn new(event_tx: mpsc::Sender<HandleMessage>) -> Self {
        Self {
            is_calling: false,
            status: RunningStatus::START,
            event_tx,
        }
    }

    pub async fn run(&mut self, mut message_rx: mpsc::Receiver<GCodeHandlerMessage>) {
        println!("gcode inside run");
        let mut g_code_interval = time::interval(Duration::from_secs(60));
        loop {
            select! {
                    _ = g_code_interval.tick() => {
                        if !self.is_calling && self.status == RunningStatus::START {
                            self.is_calling = true;
                            let generate_code_response = g_code().await;

                            match generate_code_response {
                                Ok(response) => {

                                    let _ = self.event_tx.send(HandleMessage::GenerateCodeRes {response: Ok(response.code.clone()) }).await;
                                    self.is_calling = false;

                                    println!("generateCodeResponse-response {:?}", response.code);
                              
                                }
                                Err(e) => {
                                    eprintln!("Error in generate code : {:?} ", e);
                    
                                }
                            }

                           
                        }
                    }
                    msg = message_rx.recv() => {
                        if msg.is_none() {
                            continue;
                        }

                        match msg.unwrap() {
                            GCodeHandlerMessage::ChangeRunningStatus { status } => {
                                self.status = status;
                                if status.clone() != RunningStatus::STOP { println!("continue!") };
                                break;
                            }
                        };
                    }
            }
        }
    }
}

pub enum PCodeHandlerMessage {
    ChangeRunningStatus { status: RunningStatus },
    CodeChanged { code: String },
}

struct ProvisionCodeHandler {
    is_calling: bool,
    code: Option<String>,
    status: RunningStatus,
    event_tx: mpsc::Sender<HandleMessage>,
}

impl ProvisionCodeHandler {
    pub fn new(event_tx: mpsc::Sender<HandleMessage>) -> Self {
        Self {
            is_calling: false,
            status: RunningStatus::START,
            code: None,
            event_tx,
        }
    }

    pub async fn run(&mut self, mut message_rx: mpsc::Receiver<PCodeHandlerMessage>) {
        println!("pcode inside run");
        let mut p_code_interval = time::interval(Duration::from_secs(10));
        loop {
            select! {
                    _ = p_code_interval.tick() => {
                        println!("calling PROVISIONING-----");
                        if !self.is_calling && self.status == RunningStatus::START && self.code.is_some(){
                            println!("calling PROVISIONING--IFF---");

                            self.is_calling = true;
                            let provisioning_res = p_code(self.code.clone().unwrap()).await;

                            match provisioning_res {
                                Ok(response) => {
                                    println!("p_code resp {:?} ", response);

                                    if response.success.clone() {
                                        let _ = self.event_tx.send(HandleMessage::ProvisionCodeRes {response: Ok(response.success.clone()) }).await;
                                        self.is_calling = true;
                                        break;
                                    }
                                    else {
                                        println!("PROVISION CODE NOT SUCCESS {:?} ", self.code.clone().unwrap());
                                        self.is_calling = false;
                                    }
                                },
                                Err(e) => { 
                                    println!("pc_code error {:?} ", e);
                                    self.is_calling = false;
                                }
                            }

                        }
                    }
                    msg = message_rx.recv() => {
                        if msg.is_none() {
                            continue;
                        }

                        match msg.unwrap() {
                            PCodeHandlerMessage::ChangeRunningStatus { status } => {
                                self.status = status;
                            }
                            PCodeHandlerMessage::CodeChanged {code} => {
                                self.code = Some(code);
                            }
                        };
                    }

            }
        }
    }
}


#[derive(Debug)]
pub struct GenerateCodeResp {
    pub code: String,
    pub message: String
}

pub async fn g_code() -> anyhow::Result<GenerateCodeResp> {
    let provision_manager_client_response = ProvisionManagerClient::new().await;
    let mut provision_manager_client = match provision_manager_client_response {
        Ok(r) => r,
        Err(e) => {
            bail!("Provision Handler-connect clinet error:: {}", e);
        }
    };

    let generate_code_response = provision_manager_client.generate_code().await;
    let provisioning_code: GenerateCodeResp = match generate_code_response {
        Ok(r) => {
            println!("inside g_code: {:?} ", r);
            GenerateCodeResp {
                code: r.code,
                message: String::from("")
            }
        },
        Err(e) => {
            eprintln!("Provision Handler-generate_code error:: {:?}", e);
            GenerateCodeResp {
                code: String::from(""),
                message: e.to_string()
            }
        }
    };

    // let mut provisioning_code = GenerateCodeResp{
    //     code: "KEG3EK".to_owned(), message: "".to_owned()
    // };

    println!("provisioning_code {:?} ", provisioning_code);
    Ok(provisioning_code)
}

#[derive(Debug)]
pub struct ProvisioningStatusResponse {
    pub success: bool,
    pub message: String
}

pub async fn p_code(code: String) -> anyhow::Result<ProvisioningStatusResponse>  {
    println!("p_code received {:?}", code);
    
    let provision_manager_client_response = ProvisionManagerClient::new().await;
    let mut provision_manager_client = match provision_manager_client_response {
        Ok(r) => r,
        Err(e) => {
            bail!("Provision Handler-connect clinet error:: {}", e);
        }
    };

    println!("calling PROVISIONING-FOR---- {:?} ", code);
    let provisioning_response: ProvisioningStatusResponse = match provision_manager_client.provision_by_code(code).await {
        Ok(r) => {
            ProvisioningStatusResponse {
                success: true,
                message: String::from("")

            }
        },
        Err(e) => {
            ProvisioningStatusResponse {
                success: false,
                message: e.to_string()

            }
        }
    };


    // let mut provisioning_response = ProvisioningStatusResponse{
    //     // success: true, message: "".to_owned()
    //     success: false, message: "SOmething went wrong".to_owned()
    // };

    Ok(provisioning_response)

}