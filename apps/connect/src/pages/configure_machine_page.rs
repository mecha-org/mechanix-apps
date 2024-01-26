use async_trait::async_trait;
use gtk::prelude::*;
use relm4::{component::{AsyncComponent, AsyncComponentParts}, gtk::{self, prelude::{WidgetExt, ButtonExt}, Button, glib::clone}, AsyncComponentSender, ComponentParts, ComponentSender, SimpleComponent};
use crate::settings::{Modules, WidgetConfigs};
use custom_utils::{get_gif_from_path, get_image_from_path};

pub struct Settings {
    pub modules: Modules,
    pub widget_configs: WidgetConfigs,
}

pub struct ConfigureMachinePage {
    settings: Settings,
}

#[derive(Debug)]
enum AppInput {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum ConfigureOutput {
    BackPressed,
    NextPressed
}

#[derive(Debug)]
pub enum InputMessage {
    ActiveScreen(String),
}

pub struct AppWidgets {
}

#[async_trait(?Send)]
impl AsyncComponent for ConfigureMachinePage {

    type Init = Settings;
    type Input = InputMessage;
    type Output = ConfigureOutput;
    type Root = gtk::Box;
    type Widgets = AppWidgets;
    type CommandOutput = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
        .build()
    }

    /// Initialize the UI and model.
    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let modules = init.modules.clone();
        let widget_configs = init.widget_configs.clone();

        let model = ConfigureMachinePage {settings: init};

        let main_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(["app-container", "configure-container"])
        .build();

        // let paintable = GifPaintable::new();
       
        // let bytes = include_bytes!("../../src/assets/images/machine_searching.gif");
        // let _ = paintable.load_from_bytes(bytes);

        // get gif
        let gif_path = modules.pages_settings.configure_machine.machine_searching.clone();
        let paintable = get_gif_from_path(gif_path);

        let image_from = gtk::Image::builder()
            .width_request(262)
            .height_request(262)
            .paintable(&paintable)
            .css_classes(["gif-img"])
            .build();

        let label1 = gtk::Label::builder()
        .label("Configuring")
        .css_classes(["configure-text"])
        .halign(gtk::Align::Center)
        .build();

        let label2 = gtk::Label::builder()
        .label("Configuring your machine, please wait...")
        .css_classes(["configure-text-label"])
        .halign(gtk::Align::Center)
        .build();

        main_content_box.append(&image_from);
        main_content_box.append(&label1);
        main_content_box.append(&label2);

        // TEMP: REMOVE LATER
        let footer_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .valign(gtk::Align::End)
        .css_classes(["footer-container"])
        .build();

        // footer_box
        let footer_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();
        
        let back_icon_img: gtk::Image = get_image_from_path(
            widget_configs.footer.back_icon,
            &[],
        );
        let back_button_box = gtk::Box::builder().hexpand(true).build();
        let back_button = Button::builder().build();
        back_button.set_child(Some(&back_icon_img));
        back_button.add_css_class("footer-container-button");

        back_button.connect_clicked(clone!(@strong sender => move |_| {
            let _ =  sender.output(ConfigureOutput::BackPressed);
          }));

        let next_icon_img: gtk::Image = get_image_from_path(
            widget_configs.footer.next_icon,
            &[],
        );
        let next_button = Button::new();
        next_button.set_child(Some(&next_icon_img));
        next_button.add_css_class("footer-container-button");

        next_button.connect_clicked(clone!(@strong sender => move |_| {
            let _ =  sender.output(ConfigureOutput::NextPressed);
          }));

        back_button_box.append(&back_button);
        footer_box.append(&back_button_box);
        footer_box.append(&next_button);

        footer_content_box.append(&footer_box);
        main_content_box.append(&footer_content_box);
        // TEMP: REMOVE LATER

        root.append(&main_content_box);

        let widgets = AppWidgets {  };

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) { 

    }

  
} 

async fn init_services(sender: relm4::Sender<InputMessage>) { 

}