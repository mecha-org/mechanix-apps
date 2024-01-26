use std::time::Duration;

use crate::{server::provision_client::ProvisionManagerClient, settings::{Modules, WidgetConfigs}};
use async_trait::async_trait;
use custom_utils::{get_gif_from_path, get_image_from_path};
use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts}, gtk::{
        self,
        glib::clone,
        prelude::{ButtonExt, WidgetExt},
        Button,
    }, AsyncComponentSender
};

pub struct Settings {
    pub modules: Modules,
    pub widget_configs: WidgetConfigs,
}
pub struct CheckInternetPage {
    settings: Settings,
}

#[derive(Debug)]
enum AppInput {
}

#[derive(Debug)]
pub enum InputMessage {
    ActiveScreen(String),
    NextScreen,
    ConnectionNotFound,
    ShowError
}

#[derive(Debug)]
pub enum CheckInternetOutput {
    BackPressed,
    LinkMachine,
    ConnectionNotFound,
    ShowError
}

pub struct AppWidgets {}

#[async_trait(?Send)]
impl AsyncComponent for CheckInternetPage {
    type Init = Settings;
    type Input = InputMessage;
    type Output = CheckInternetOutput;
    type Root = gtk::Box;
    type Widgets = AppWidgets;
    type CommandOutput = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder().build()
    }

    /// Initialize the UI and model.
    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let modules = init.modules.clone();
        let widget_configs = init.widget_configs.clone();

        let model = CheckInternetPage { settings: init };

        let main_content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["app-container", "check-internet-text"])
            .build();

        let footer_content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .valign(gtk::Align::End)
            .css_classes(["footer-container"])
            .build();

        // get gif
        let gif_path = modules.pages_settings.check_internet.search_wifi.clone();
        let paintable = get_gif_from_path(gif_path);

        let image_from = gtk::Image::builder()
            .width_request(290)
            .height_request(290)
            .paintable(&paintable)
            .css_classes(["gif-img"])
            .build();

        let label1: gtk::Label = gtk::Label::builder()
            .label("Checking for internet connectivity...")
            .build();

        let label2: gtk::Label = gtk::Label::builder().label("Please wait").build();

        main_content_box.append(&image_from);
        main_content_box.append(&label1);
        main_content_box.append(&label2);

        let footer_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let back_icon_img: gtk::Image = get_image_from_path(widget_configs.footer.back_icon, &[]);
        let back_button_box = gtk::Box::builder().hexpand(true).build();
        let back_button = Button::builder().build();
        back_button.set_child(Some(&back_icon_img));
        back_button.add_css_class("footer-container-button");

        back_button.connect_clicked(clone!(@strong sender => move |_| {
          let _ =  sender.output(CheckInternetOutput::BackPressed);
        }));

        let next_icon_img: gtk::Image = get_image_from_path(widget_configs.footer.next_icon, &[]);
        let next_button = Button::new();
        next_button.set_child(Some(&next_icon_img));
        next_button.add_css_class("footer-container-button");

        next_button.connect_clicked(clone!(@strong sender => move |_| {
          let _ =  sender.output(CheckInternetOutput::LinkMachine);
        }));

        back_button_box.append(&back_button);
        footer_box.append(&back_button_box);
        footer_box.append(&next_button);

        footer_content_box.append(&footer_box);
        main_content_box.append(&footer_content_box);

        root.append(&main_content_box);

        let widgets = AppWidgets {};

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) { 
        match message {
            InputMessage::ActiveScreen(text) => {
                println!("active screen: {:?}", text);
                let sender: relm4::Sender<InputMessage> = sender.input_sender().clone();
                let _ = init_services(sender).await;
            },
            InputMessage::NextScreen => {
                let _ =  sender.output(CheckInternetOutput::LinkMachine);
            },
            InputMessage::ConnectionNotFound => {
                let _ =  sender.output(CheckInternetOutput::ConnectionNotFound);
            }
            InputMessage::ShowError => {
                let _ =  sender.output(CheckInternetOutput::ShowError);
            }
        }
        
    }
}

async fn init_services(sender: relm4::Sender<InputMessage>) {
    println!("init services called...");

    let time_duration=Duration::from_millis(7000);
    let _ = tokio::time::sleep(time_duration);

    match ProvisionManagerClient::new().await {
        Ok(mut client) => {
            match client.ping().await {
                Ok(response) => {
                    println!("ping response {:?} ", response);
                   if response.code == "success" {
                    let _ = sender.send(InputMessage::NextScreen);
                   }
                   else {
                    let _ = sender.send(InputMessage::ConnectionNotFound);
                   }
                },
                Err(error) => eprintln!("ping error: {}", error)
            }
        },
        Err(error) => {
            println!("Client error :: {} ", error);
            // server connection refused! - Handler error screen
        }
    };
}
