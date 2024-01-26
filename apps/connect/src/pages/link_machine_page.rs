use async_trait::async_trait;
use custom_utils::get_image_from_path;
use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts},
    gtk::{
        self,
        gdk::Display,
        glib::clone,
        prelude::{ButtonExt, WidgetExt},
        Button, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION,
    },
    AsyncComponentSender,
};
use tokio::sync::oneshot;
use std::time::{Duration, Instant};

use crate::{handlers::provision::handler::LinkMachineHandler,settings::WidgetConfigs};

// use crate::{services::provisionHandler::ProvisionHandler, settings::WidgetConfigs};

pub struct Settings {
    pub widget_configs: WidgetConfigs,
}

pub struct LinkMachinePage {
    settings: Settings,
    connect_code: String,
    timer: i32,
    provision_status: bool,
    progress: f64,
    progress_angle: i32,
    current_time: i32,
}

#[derive(Debug)]
enum AppInput {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum LinkMachineOutput {
    BackPressed,
    NextPressed,
    ShowError
}

#[derive(Debug)]
pub enum InputMessage {
    ActiveScreen(String),
    CodeChanged(String),
    GenerateCodeError(String),
    ProvisionSuccess,
    ShowError(String),
    TimeTick,
}

pub struct AppWidgets {
    connect_code_label: gtk::Label,
    spinner: gtk::Spinner,
    // timer_label: gtk::Label,
    // time_bar: gtk::ProgressBar,
}

// //
pub struct LinkMachineData {
    pub generated_code: String,
}

const TIMER: i32 = 10;

#[async_trait(?Send)]
impl AsyncComponent for LinkMachinePage {
    type Init = Settings;
    type Input = InputMessage;
    type Output = LinkMachineOutput;
    type Root = gtk::Box;
    type Widgets = AppWidgets;
    type CommandOutput = ();

    fn init_root() -> Self::Root {
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("../assets/css/style.css"));
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        gtk::Box::builder().build()
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let widget_configs = init.widget_configs.clone();

        let main_content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["app-container"])
            .build();

        let footer_content_box: gtk::Box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .valign(gtk::Align::End)
            .css_classes(["footer-container"])
            .build();

        let header_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["link-machine-header"])
            .build();

        let header_label = gtk::Label::builder()
            .label("Linking your machine")
            .halign(gtk::Align::Start)
            .build();
        header_box.append(&header_label);
        main_content_box.append(&header_box);

        let header_label_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let header_p = gtk::Label::builder()
            .label("Use the below code to onnect this machine to your mech.so \naccount")
            .css_classes(["link-machine-header-label"])
            .build();

        header_label_box.append(&header_p);
        main_content_box.append(&header_label_box);

        // check-code
        let main_code_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .css_classes(["link-machine-border-box"])
            .build();

        let code_label_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        // spinner
        let spinner = gtk::Spinner::builder()
            .css_classes(["blue"])
            .height_request(30)
            .width_request(30)
            .build();
        spinner.set_spinning(false);

        // progressbar
        let time_bar = gtk::ProgressBar::builder().build();
        time_bar.style_context().add_class("time-progressbar");
        time_bar.set_visible(false);

        let connect_code_label = gtk::Label::builder()
            .label("") // ABCD 1234
            .css_classes(["link-machine-code"])
            .build();

        // let timer_label = gtk::Label::builder()
        //     .label("")
        //     .css_classes(["link-machine-code"])
        //     .build();

        // .css_classes(["link-machine-code", "custom-circle"])

        code_label_box.append(&connect_code_label);
        main_code_box.append(&code_label_box);
        main_code_box.append(&time_bar);
        main_code_box.append(&spinner);
        // main_code_box.append(&timer_label);

        main_content_box.append(&main_code_box);

        let main_steps_box: gtk::Box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["link-machine-steps-container"])
            .build();

        let linking_step1_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["link-machine-steps-box"])
            .hexpand(true)
            .build();

        let step1_label_box = gtk::Box::builder()
            .css_classes(["square-border-box"])
            .valign(gtk::Align::Start)
            .build();

        let step1_label = gtk::Label::builder()
            .label("1")
            .width_request(10)
            .height_request(10)
            .build();
        step1_label_box.append(&step1_label);

        let step1_text = gtk::Label::builder()
            .label("Sign up on mecha.so")
            .css_classes(["link-machine-steps-text"])
            .build();

        linking_step1_box.append(&step1_label_box);
        linking_step1_box.append(&step1_text);

        main_steps_box.append(&linking_step1_box);

        //
        let linking_step2_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["link-machine-steps-box"])
            .hexpand(true)
            .build();

        let step2_label_box = gtk::Box::builder()
            .css_classes(["square-border-box"])
            .valign(gtk::Align::Start)
            .build();

        let step2_label = gtk::Label::builder()
            .label("2")
            .width_request(10)
            .height_request(10)
            .build();
        step2_label_box.append(&step2_label);

        let step2_text = gtk::Label::builder()
            .label("Use the Console app or developer CLI to add a new \nmachine")
            .css_classes(["link-machine-steps-text"])
            .build();

        linking_step2_box.append(&step2_label_box);
        linking_step2_box.append(&step2_text);

        main_steps_box.append(&linking_step2_box);

        //
        let linking_step3_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["link-machine-steps-box"])
            .hexpand(true)
            .build();

        let step3_label_box = gtk::Box::builder()
            .css_classes(["square-border-box"])
            .valign(gtk::Align::Start)
            .build();

        let step3_label = gtk::Label::builder()
            .label("3")
            .width_request(10)
            .height_request(10)
            .build();
        step3_label_box.append(&step3_label);

        let step3_text = gtk::Label::builder()
            .label("Use this code when asked to enter the provisioning code")
            .css_classes(["link-machine-steps-text"])
            .build();

        linking_step3_box.append(&step3_label_box);
        linking_step3_box.append(&step3_text);

        main_steps_box.append(&linking_step3_box);

        main_content_box.append(&main_steps_box);

        // footer_box
        let footer_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .valign(gtk::Align::End)
            .build();

        let back_icon_img: gtk::Image = get_image_from_path(widget_configs.footer.back_icon, &[]);
        let back_button_box = gtk::Box::builder().hexpand(true).build();
        let back_button = gtk::Button::builder().build();
        back_button.set_child(Some(&back_icon_img));
        back_button.add_css_class("footer-container-button");

        back_button.connect_clicked(clone!(@strong sender => move |_| {
          let _ =  sender.output(LinkMachineOutput::BackPressed);
        }));

        // TEMP : REMVOE THIS LATER - NOT IN UI
        let next_icon_img: gtk::Image = get_image_from_path(widget_configs.footer.next_icon, &[]);
        let next_button = Button::new();
        next_button.set_child(Some(&next_icon_img));
        next_button.add_css_class("footer-container-button");

        next_button.connect_clicked(clone!(@strong sender => move |_| {
          let _ =  sender.output(LinkMachineOutput::NextPressed);
        }));

        back_button_box.append(&back_button);
        footer_box.append(&back_button_box);

        // TEMP : REMVOE THIS LATER
        footer_box.append(&next_button);

        footer_content_box.append(&footer_box);
        main_content_box.append(&footer_content_box);

        root.append(&main_content_box);

        let model = LinkMachinePage {
            settings: init,
            connect_code: "".to_string(),
            timer: TIMER,
            provision_status: false,
            progress: 0.0,
            progress_angle: 0,
            current_time: 0,
        };

        let widgets = AppWidgets {
            connect_code_label,
            spinner,
            // time_bar,
            // timer_label,
        };

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // println!("Inside update {:?}", message);
        let seconds = Duration::from_secs(10);
        let start = Instant::now();

        match message {
            InputMessage::ActiveScreen(text) => {
                let sender: relm4::Sender<InputMessage> = sender.input_sender().clone();
                let result = init_services(sender).await;
            },
            InputMessage::ProvisionSuccess => {
             let _ =  sender.output(LinkMachineOutput::NextPressed);
            },
            InputMessage::CodeChanged(code) => {
                println!("inside InputMessage code change");
                self.connect_code = code.clone();
            },
            InputMessage::TimeTick => {
                // let mut current_time = self.timer.clone();
                // if current_time >= 0 {
                //     if current_time == 0 {
                //         current_time = TIMER.clone();
                //         // // Generate code
                //         let _ = sender.input_sender().send(InputMessage::GenerateCode);
                //     }
                //     current_time -= 1;
                // }
                // self.timer = current_time;
                // // println!("NEW TIME {:?} & OG TIMER {:?}", self.timer, TIMER);

                // println!("After TIme's up!! {:?} & {:?}", self.connect_code, self.timer);
                // let _ = sender.input_sender().send(InputMessage::GenerateCode);

                // if self.timer > 0 {
                //     self.timer -= 1;
                //     println!("Remaining Time: {:?}", self.timer);
                // } else {
                //     self.timer = TIMER.clone();
                //     let _ = sender.input_sender().send(InputMessage::GenerateCode);
                // }
            },
            InputMessage::GenerateCodeError(error) => {
                println!("Generate code error: {:?} ", error);
                println!("SHOW TOAST!");
            },
            InputMessage::ShowError(text) => {
                println!("Error to be shown:: {:?} ", text);
                let _ =  sender.output(LinkMachineOutput::ShowError);
            }
            
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: AsyncComponentSender<Self>) {
        println!("update_view {:?} ", self.connect_code);
        widgets.connect_code_label.set_label(&self.connect_code);
        widgets.spinner.set_spinning(true);

    }
  
}

async fn init_services(sender: relm4::Sender<InputMessage>) {
    println!("init services called..."); 

    let sender_clone_1 = sender.clone();
    let mut link_machine_handler = LinkMachineHandler::new();
    let _ = relm4::spawn_local(async move {
        let _ = link_machine_handler.run(sender_clone_1).await;
    });
}
