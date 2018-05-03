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
    Toggle(Options),
    Generate,
    Save,
    Quit,
}

pub struct Win {
    model: Model,
    window: gtk::Window,
    container: ContainerOptions,
    amount: AmountOptions,
    variant: VariantOptions,
    rotation: RotationOptions,
    problem_tv: gtk::TextView,
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
            Msg::Toggle(c) => match c {
                Options::Container => self.container.toggle(),
                Options::Amount => self.amount.toggle(),
                Options::Variant => self.variant.toggle(),
                Options::Rotation => self.rotation.toggle(),
            },
            Msg::Generate => {
                let mut generator = Generator::new();
                if !self.container.switch.get_active() {
                    let width =
                        self.container.width_spinbtn.get_value_as_int() as u32;
                    let height =
                        self.container.height_spinbtn.get_value_as_int() as u32;
                    generator.container(Rectangle::new(width, height));
                }

                if !self.amount.switch.get_active() {
                    let amount =
                        self.amount.spinbtn.get_value_as_int() as usize;
                    generator.rectangles(amount);
                }

                if !self.variant.switch.get_active() {
                    let v = if self.variant.fixed_radio.get_active() {
                        Variant::Fixed(0)
                    } else {
                        Variant::Free
                    };

                    generator.variant(v);
                }

                if !self.rotation.switch.get_active() {
                    let r = self.rotation.checkbtn.get_active();
                    generator.allow_rotation(r);
                }

                let problem = generator.generate();
                let problem_text = problem.digest();

                self.model.problem = Some(problem);
                self.problem_tv
                    .get_buffer()
                    .expect("couldn't get buffer")
                    .set_text(&problem_text);
            }
            Msg::Save => {
                let dialog = gtk::FileChooserDialog::new(
                    Some("Save File"),
                    Some(&self.window),
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
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
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

        let container = ContainerOptions::from_builder(&builder);
        connect!(
            relm,
            container.switch,
            connect_toggled(_),
            Msg::Toggle(Options::Container)
        );

        let amount = AmountOptions::from_builder(&builder);
        connect!(
            relm,
            amount.switch,
            connect_toggled(_),
            Msg::Toggle(Options::Amount)
        );

        let variant = VariantOptions::from_builder(&builder);
        connect!(
            relm,
            variant.switch,
            connect_toggled(_),
            Msg::Toggle(Options::Variant)
        );

        let rotation = RotationOptions::from_builder(&builder);
        connect!(
            relm,
            rotation.switch,
            connect_toggled(_),
            Msg::Toggle(Options::Rotation)
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
            window,
            container,
            amount,
            variant,
            rotation,
            problem_tv,
        }
    }
}

pub enum Options {
    Container,
    Amount,
    Variant,
    Rotation,
}

struct ContainerOptions {
    switch: gtk::CheckButton,
    filters_box: gtk::Box,
    width_spinbtn: gtk::SpinButton,
    height_spinbtn: gtk::SpinButton,
}

impl ContainerOptions {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let switch = builder.get_object("container_btn").unwrap();
        let filters_box = builder
            .get_object("container_filter_box")
            .unwrap();
        let width_spinbtn = builder
            .get_object("container_width_spinbtn")
            .unwrap();
        let height_spinbtn = builder
            .get_object("container_height_spinbtn")
            .unwrap();

        ContainerOptions {
            switch,
            filters_box,
            width_spinbtn,
            height_spinbtn,
        }
    }

    fn toggle(&mut self) {
        self.filters_box
            .set_sensitive(!self.switch.get_active());
    }
}

struct AmountOptions {
    switch: gtk::CheckButton,
    spinbtn: gtk::SpinButton,
}

impl AmountOptions {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let switch = builder.get_object("amount_btn").unwrap();
        let spinbtn = builder.get_object("amount_spinbtn").unwrap();
        AmountOptions {
            switch,
            spinbtn,
        }
    }

    fn toggle(&mut self) {
        self.spinbtn
            .set_sensitive(!self.switch.get_active());
    }
}

struct VariantOptions {
    switch: gtk::CheckButton,
    btn_box: gtk::ButtonBox,
    fixed_radio: gtk::RadioButton,
}

impl VariantOptions {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let switch = builder.get_object("variant_btn").unwrap();
        let btn_box = builder.get_object("variant_btn_box").unwrap();
        let fixed_radio = builder
            .get_object("variant_fixed_rbtn")
            .unwrap();
        let _free_radio: gtk::RadioButton =
            builder.get_object("variant_free_rbtn").unwrap();

        VariantOptions {
            switch,
            btn_box,
            fixed_radio,
        }
    }

    fn toggle(&mut self) {
        self.btn_box
            .set_sensitive(!self.switch.get_active());
    }
}

struct RotationOptions {
    switch: gtk::CheckButton,
    checkbtn: gtk::CheckButton,
}

impl RotationOptions {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let switch = builder.get_object("rotation_btn").unwrap();
        let checkbtn = builder.get_object("rotation_checkbtn").unwrap();

        RotationOptions {
            switch,
            checkbtn,
        }
    }

    fn toggle(&mut self) {
        self.checkbtn
            .set_sensitive(!self.switch.get_active());
    }
}
