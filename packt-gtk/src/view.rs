use gtk::{self, prelude::*};
use packt_core::domain;
use packt_core::domain::problem::{Generator, Variant};
use packt_core::domain::Rectangle;
use relm::{Relm, Update, Widget};
use std;

pub struct Model {
    problem: Option<domain::Problem>,
}

#[derive(Msg)]
pub enum Msg {
    Toggle(Setting),
    Generate,
    Save,
    Quit,
}

struct Widgets {
    window: gtk::Window,
    settings: SettingsPanel,
    problem_tv: gtk::TextView,
    save_btn: gtk::Button,
}

pub struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model { problem: None }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Toggle(c) => self.widgets.settings.toggle(c),
            Msg::Generate => self.generate_problem(),
            Msg::Save => self.save_problem(),
            Msg::Quit => gtk::main_quit(),
        }
    }
}


impl Widget for Win {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        use self::Setting::*;

        let glade_src = include_str!("packt.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let window: gtk::Window = builder
            .get_object("generator_window")
            .expect("couldn't get main_window");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let settings = SettingsPanel::from_builder(&builder);

        connect!(
            relm,
            settings.container_switch,
            connect_toggled(_),
            Msg::Toggle(Container)
        );

        connect!(
            relm,
            settings.amount_switch,
            connect_toggled(_),
            Msg::Toggle(Amount)
        );

        connect!(
            relm,
            settings.variant_switch,
            connect_toggled(_),
            Msg::Toggle(Variant)
        );

        connect!(
            relm,
            settings.rotation_switch,
            connect_toggled(_),
            Msg::Toggle(Rotation)
        );

        let generate_btn: gtk::Button = builder
            .get_object("generate_button")
            .expect("couldn't get generate_button");
        connect!(
            relm,
            generate_btn,
            connect_clicked(_),
            Msg::Generate
        );

        let save_btn: gtk::Button = builder
            .get_object("save_button")
            .expect("couldn't get save_button");
        connect!(relm, save_btn, connect_clicked(_), Msg::Save);

        let problem_tv: gtk::TextView = builder
            .get_object("problem_textview")
            .expect("couldn't get problem_textview");

        window.show_all();

        Win {
            model,
            widgets: Widgets {
                window,
                settings,
                problem_tv,
                save_btn,
            },
        }
    }
}

pub enum Setting {
    Container,
    Amount,
    Variant,
    Rotation,
}

struct SettingsPanel {
    container_switch: gtk::CheckButton,
    container_filters_box: gtk::Box,
    container_width_spinbtn: gtk::SpinButton,
    container_height_spinbtn: gtk::SpinButton,
    amount_switch: gtk::CheckButton,
    amount_spinbtn: gtk::SpinButton,
    variant_switch: gtk::CheckButton,
    variant_btn_box: gtk::ButtonBox,
    variant_fixed_radio: gtk::RadioButton,
    rotation_switch: gtk::CheckButton,
    rotation_checkbtn: gtk::CheckButton,
}

impl SettingsPanel {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let container_switch = builder.get_object("container_btn").unwrap();
        let container_filters_box = builder
            .get_object("container_filter_box")
            .unwrap();
        let container_width_spinbtn = builder
            .get_object("container_width_spinbtn")
            .unwrap();
        let container_height_spinbtn = builder
            .get_object("container_height_spinbtn")
            .unwrap();
        let amount_switch = builder.get_object("amount_btn").unwrap();
        let amount_spinbtn = builder.get_object("amount_spinbtn").unwrap();
        let variant_switch = builder.get_object("variant_btn").unwrap();
        let variant_btn_box = builder.get_object("variant_btn_box").unwrap();
        let variant_fixed_radio = builder
            .get_object("variant_fixed_rbtn")
            .unwrap();
        let _free_radio: gtk::RadioButton =
            builder.get_object("variant_free_rbtn").unwrap();
        let rotation_switch = builder.get_object("rotation_btn").unwrap();
        let rotation_checkbtn =
            builder.get_object("rotation_checkbtn").unwrap();

        SettingsPanel {
            container_switch,
            container_filters_box,
            container_width_spinbtn,
            container_height_spinbtn,
            amount_switch,
            amount_spinbtn,
            variant_switch,
            variant_btn_box,
            variant_fixed_radio,
            rotation_switch,
            rotation_checkbtn,
        }
    }

    fn toggle(&mut self, o: Setting) {
        use self::Setting::*;
        match o {
            Container => self.container_filters_box
                .set_sensitive(!self.container_switch.get_active()),
            Amount => self.amount_spinbtn
                .set_sensitive(!self.amount_switch.get_active()),
            Variant => self.variant_btn_box
                .set_sensitive(!self.variant_switch.get_active()),
            Rotation => self.rotation_checkbtn
                .set_sensitive(!self.rotation_switch.get_active()),
        }
    }
}

impl Win {
    fn save_problem(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Save File"),
            Some(&self.widgets.window),
            gtk::FileChooserAction::Save,
        );

        let cancel: i32 = gtk::ResponseType::Cancel.into();
        let accept: i32 = gtk::ResponseType::Accept.into();
        dialog.add_button("Cancel", cancel);
        dialog.add_button("Save", accept);

        if let Ok(p) = std::env::current_dir() {
            dialog.set_current_folder(p);
        } else if let Some(p) = std::env::home_dir() {
            dialog.set_current_folder(p);
        }

        if accept == dialog.run() {
            if let Some(path) = dialog.get_filename() {
                self.model
                    .problem
                    .as_ref()
                    .unwrap()
                    .save(path)
                    .unwrap();
            }
        }
        dialog.close();
    }

    fn generate_problem(&mut self) {
        let settings = &self.widgets.settings;

        self.widgets.save_btn.set_sensitive(true);
        let mut generator = Generator::new();
        if !settings.container_switch.get_active() {
            let width = settings
                .container_width_spinbtn
                .get_value_as_int() as u32;
            let height = settings
                .container_height_spinbtn
                .get_value_as_int() as u32;
            generator.container(Rectangle::new(width, height));
        }

        if !settings.amount_switch.get_active() {
            let amount = settings.amount_spinbtn.get_value_as_int() as usize;
            generator.rectangles(amount);
        }

        if !settings.variant_switch.get_active() {
            let v = if settings.variant_fixed_radio.get_active() {
                Variant::Fixed(0)
            } else {
                Variant::Free
            };

            generator.variant(v);
        }

        if !settings.rotation_switch.get_active() {
            let r = settings.rotation_checkbtn.get_active();
            generator.allow_rotation(r);
        }

        let problem = generator.generate();
        let problem_text = problem.digest();
        self.model.problem = Some(problem);
        self.widgets
            .problem_tv
            .get_buffer()
            .expect("couldn't get buffer")
            .set_text(&problem_text);
    }
}