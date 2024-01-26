mod pages;
mod settings;
pub mod errors;
mod server; 
mod handlers;

use async_trait::async_trait;
use gtk::prelude::{BoxExt, GtkWindowExt};
use pages::{
    app_info_page::{AppInfoOutput, AppInfoPage, Settings as AppInfoSettings},
    check_internet_page::{
        CheckInternetOutput, CheckInternetPage, Settings as CheckInternetSettings,
    },
    configure_machine_page::{
        ConfigureMachinePage, ConfigureOutput, Settings as ConfigureMachineSettings,
    },
    device_info_page::{DeviceInfoPage, DevicePageOutput, Settings as DeviceInfoSettings},
    link_machine_page::{LinkMachineOutput, LinkMachinePage, Settings as LinkMachineSettings},
    no_internet_page::{NoInternetPage, PageOutput, Settings as NoInternetSettings},
    setup_failed_page::{Settings as SetupFailedSettings, SetupFailOutput, SetupFailedPage},
    setup_success_page::{Settings as SetupSuccessSettings, SetupSuccessOutput, SetupSuccessPage},
};
use relm4::{gtk::glib::clone, component::{AsyncComponentParts, AsyncComponent, AsyncComponentController, AsyncController}, AsyncComponentSender};
use relm4::{gtk, ComponentController};
use relm4::{Component, Controller, RelmApp};
use settings::ScreenSettings;
use std::{fmt};
use tracing::info;

struct MechaConnectApp {
    current_page: Pages,
    pages_stack: gtk::Stack,
    link_machine_page: AsyncController<LinkMachinePage>,
    check_internet_page: AsyncController<CheckInternetPage>,
    configure_machine_page: AsyncController<ConfigureMachinePage>
}

#[derive(Debug)]
enum Pages {
    AppInfoPage,
    CheckInternetPage,
    NoInternetPage,
    LinkMachinePage,
    ConfigureMachinePage,
    SetupSuccessPage,
    SetupFailedPage,
    DeviceInfoPage,
}

impl fmt::Display for Pages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pages::AppInfoPage => write!(f, "app_info_page"),
            Pages::CheckInternetPage => write!(f, "check_internet_page"),
            Pages::NoInternetPage => write!(f, "no_internet_page"),
            Pages::LinkMachinePage => write!(f, "link_machine_page"),
            Pages::ConfigureMachinePage => write!(f, "configure_machine_page"),
            Pages::SetupSuccessPage => write!(f, "setup_success_page"),
            Pages::SetupFailedPage => write!(f, "setup_failed_page"),
            Pages::DeviceInfoPage => write!(f, "device_info_page"),
        }
    }
}

#[derive(Debug)]
enum Message {
    ChangeScreen(Pages),
    // NextPressed
}
#[derive(Debug)]
enum AppInput {}

struct AppWidgets {
    pages_stack: gtk::Stack,
}

fn init_window(settings: ScreenSettings) -> gtk::Window {
    let window_settings = settings.window;
    let window = gtk::Window::builder()
        .title("Mecha Connect")
        .default_width(window_settings.size.0)
        .default_height(window_settings.size.1)
        .css_classes(["window"])
        .build();
    window
}

#[async_trait(?Send)]
impl AsyncComponent for MechaConnectApp {
    type Input = Message;
    type Output = ();
    type Init = ();
    type Root = gtk::Window;
    type Widgets = AppWidgets;
    type CommandOutput = Message;

    fn init_root() -> Self::Root {
        let settings = match settings::read_settings_yml() {
            Ok(settings) => settings,
            Err(_) => ScreenSettings::default(),
        };

        info!(
            task = "initalize_settings",
            "settings initialized for Lock Screen: {:?}", settings
        );

        let window = init_window(settings);
        window
    }

    /// Initialize the UI and model.
    async fn init(
        _: Self::Init,
        window: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {

        let settings = match settings::read_settings_yml() {
            Ok(settings) => settings,
            Err(_) => ScreenSettings::default(),
        };

        let css = settings.css.clone();
        relm4::set_global_css_from_file(css.default);

        let modules = settings.modules.clone();
        let widget_configs = settings.widget_configs.clone();


        // // HERE grpc call

        // let generate_code_data = get_provision_data().await;

        let app_info_page: Controller<AppInfoPage> = AppInfoPage::builder()
            .launch(AppInfoSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    AppInfoOutput::BackPressed => Message::ChangeScreen(Pages::AppInfoPage),
                    AppInfoOutput::NextPressed => Message::ChangeScreen(Pages::CheckInternetPage)
                }),
            );

        let check_internet_page = CheckInternetPage::builder()
            .launch(CheckInternetSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    CheckInternetOutput::BackPressed => Message::ChangeScreen(Pages::AppInfoPage),
                    CheckInternetOutput::LinkMachine => Message::ChangeScreen(Pages::LinkMachinePage),
                    CheckInternetOutput::ConnectionNotFound => Message::ChangeScreen(Pages::NoInternetPage),
                    CheckInternetOutput::ShowError => Message::ChangeScreen(Pages::SetupFailedPage),
                }),
            );

        let no_internet_page: Controller<NoInternetPage> = NoInternetPage::builder()
            .launch(NoInternetSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    PageOutput::BackPressed => Message::ChangeScreen(Pages::CheckInternetPage),
                    // PageOutput::NextPressed => { return Message::NextPressed}
                    PageOutput::NextPressed => Message::ChangeScreen(Pages::LinkMachinePage)
                }),
            );

        let link_machine_page = LinkMachinePage::builder().launch(LinkMachineSettings{
            widget_configs: widget_configs.clone()
        })
        .forward(
            sender.input_sender(),
            clone!(@strong modules => move|msg| match msg {
                LinkMachineOutput::BackPressed => Message::ChangeScreen(Pages::NoInternetPage),
                LinkMachineOutput::NextPressed => Message::ChangeScreen(Pages::ConfigureMachinePage),
                LinkMachineOutput::ShowError => Message::ChangeScreen(Pages::SetupFailedPage),
            }),
        );

        let configure_machine_page = ConfigureMachinePage::builder()
            .launch(ConfigureMachineSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    ConfigureOutput::BackPressed => Message::ChangeScreen(Pages::LinkMachinePage),
                    ConfigureOutput::NextPressed => Message::ChangeScreen(Pages::SetupSuccessPage)
                }),
            );

        let setup_success_page = SetupSuccessPage::builder().launch(SetupSuccessSettings {
            modules: modules.clone(),
            widget_configs: widget_configs.clone()
        })
        .forward(
            sender.input_sender(),
            clone!(@strong modules => move|msg| match msg {
                SetupSuccessOutput::BackPressed => Message::ChangeScreen(Pages::ConfigureMachinePage),
                SetupSuccessOutput::NextPressed => Message::ChangeScreen(Pages::SetupFailedPage)
            }),
        );

        let setup_failed_page = SetupFailedPage::builder()
            .launch(SetupFailedSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    SetupFailOutput::BackPressed => Message::ChangeScreen(Pages::SetupSuccessPage),
                    SetupFailOutput::NextPressed => Message::ChangeScreen(Pages::DeviceInfoPage)
                }),
            );

        let device_info_page = DeviceInfoPage::builder()
            .launch(DeviceInfoSettings {
                modules: modules.clone(),
                widget_configs: widget_configs.clone(),
            })
            .forward(
                sender.input_sender(),
                clone!(@strong modules => move|msg| match msg {
                    DevicePageOutput::BackPressed => Message::ChangeScreen(Pages::SetupFailedPage),
                    DevicePageOutput::NextPressed => Message::ChangeScreen(Pages::DeviceInfoPage)
                }),
            );

        let pages_stack = gtk::Stack::builder().build();

        pages_stack.add_named(
            app_info_page.widget(),
            Option::from(Pages::AppInfoPage.to_string().as_str()),
        );

        pages_stack.add_named(
            check_internet_page.widget(),
            Option::from(Pages::CheckInternetPage.to_string().as_str()),
        );

        pages_stack.add_named(
            no_internet_page.widget(),
            Option::from(Pages::NoInternetPage.to_string().as_str()),
        );

        pages_stack.add_named(
            link_machine_page.widget(),
            Option::from(Pages::LinkMachinePage.to_string().as_str()),
        );

        pages_stack.add_named(
            configure_machine_page.widget(),
            Option::from(Pages::ConfigureMachinePage.to_string().as_str()),
        );

        pages_stack.add_named(
            setup_success_page.widget(),
            Option::from(Pages::SetupSuccessPage.to_string().as_str()),
        );

        pages_stack.add_named(
            setup_failed_page.widget(),
            Option::from(Pages::SetupFailedPage.to_string().as_str()),
        );

        pages_stack.add_named(
            device_info_page.widget(),
            Option::from(Pages::DeviceInfoPage.to_string().as_str()),
        );

        let current_page = Pages::AppInfoPage;   // OG
        // let current_page = Pages::LinkMachinePage;

        //Setting current active screen in stack
        pages_stack.set_visible_child_name(&current_page.to_string());

        // add pages here
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .hexpand(true)
            .build();

        vbox.append(&pages_stack);

        let model = MechaConnectApp { current_page, 
            pages_stack:pages_stack.clone(),
            link_machine_page,
            check_internet_page,
            configure_machine_page 
        };

        window.set_child(Some(&vbox));
        let widgets = AppWidgets { pages_stack };

        // ComponentParts { model, widgets }
        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        println!("{:?}", message);

        match message {Message::ChangeScreen(page)=>{
            __self.current_page=page;

            match self.current_page {
                Pages::AppInfoPage => {},
                Pages::CheckInternetPage => {
                    let _ = __self.check_internet_page.sender().send(pages::check_internet_page::InputMessage::ActiveScreen(self.current_page.to_string()));
                },
                Pages::NoInternetPage => {},
                Pages::LinkMachinePage => {
                    println!("THIS  IS  link_machine_page : {:?} ", self.current_page);

                    // active screen
                    // let _ = __self.link_machine_page.sender().send(pages::link_machine_page::InputMessage::ActiveScreen(self.current_page));

                    let _ = __self.link_machine_page.sender().send(pages::link_machine_page::InputMessage::ActiveScreen(self.current_page.to_string()));
                },
                Pages::ConfigureMachinePage => {
                    println!("ConfigureMachinePage ");
                    let _ = __self.configure_machine_page.sender().send(pages::configure_machine_page::InputMessage::ActiveScreen(self.current_page.to_string()));
                },
                Pages::SetupSuccessPage => {},
                Pages::SetupFailedPage => {},
                Pages::DeviceInfoPage => {},
            }
                        

          

            // let _ = __self.link_machine_page.sender().send(pages::link_machine_page::InputMessage::GenerateCodeRequest);
        
        
        }

     }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: AsyncComponentSender<Self>) {
        widgets
            .pages_stack
            .set_visible_child_name(self.current_page.to_string().as_str());
    }
}

#[tokio::main]
async fn main() {
    let app = RelmApp::new("mecha.connect.app");

    app.run_async::<MechaConnectApp>(());
}
