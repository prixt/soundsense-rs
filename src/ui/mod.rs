use gtk::prelude::*;

use crate::message::*;

pub fn build_ui(
	app: &gtk::Application,
	sound_tx: std::sync::mpsc::Sender<SoundMessage>,
	rx: glib::Receiver<UIMessage>,
) {
	let window = gtk::ApplicationWindowBuilder::new()
		.application(app)
		.title("SoundSense-rs")
		.border_width(10)
		.default_width(400)
		.default_height(400)
		.build();
	
	let accel_group = gtk::AccelGroup::new();
	window.add_accel_group(&accel_group);

	let menu_bar = gtk::MenuBarBuilder::new()
		.name("menu_bar")
		.build();
	
	let gamelog = make_menu_item("_Connect gamelog.txt", &accel_group, gdk::ModifierType::CONTROL_MASK, &['C']);
	gamelog.connect_activate({
		let sound_tx = sound_tx.clone();
		move |_| {
			let dialog = make_gamelog_dialog();
			dialog.show();
			if dialog.run() == gtk::ResponseType::Accept {
				let path = dialog.get_filename().map(|path| {
					path.to_owned()
				});
				if let Some(path) = path {
					println!("{:?}", path);
					sound_tx.send(SoundMessage::ChangeGamelog(path)).unwrap();
				}
				dialog.close();
			} else {
				dialog.close();
			}
		}
	});
	
	let soundpack = make_menu_item("_Load Soundpack", &accel_group, gdk::ModifierType::CONTROL_MASK, &['S']);
	soundpack.connect_activate({
		let sound_tx = sound_tx.clone();
		move |_| {
			let dialog = make_soundpack_dialog();
			dialog.show();
			if dialog.run() == gtk::ResponseType::Accept {
				let path = dialog.get_filename().map(|path| {
					path.to_owned()
				});
				if let Some(path) = path {
					sound_tx.send(SoundMessage::ChangeSoundpack(path)).unwrap();
				}
				dialog.close();
			} else {
				dialog.close();
			}
		}
	});

	menu_bar.append(&gamelog);
	menu_bar.append(&soundpack);
	
	let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
	v_box.pack_start(&menu_bar, false, true, 0);

	let place_holder = gtk::LabelBuilder::new()
		.label("Connect with gamelog.txt & Load soundpack.")
		.has_focus(false)
		.halign(gtk::Align::Fill)
		.valign(gtk::Align::Fill)
		.hexpand(true)
		.build();
	
	v_box.pack_start(&place_holder, true, true, 0);
	window.add(&v_box);
	
	rx.attach(None, move |message| {
		match message {
			UIMessage::ChannelNames(name) => {
				let v_box = &v_box;
				for widget in v_box.get_children() {
					WidgetExt::get_name(&widget)
						.and_then(|name| {
							if name.as_str() != "menu_bar" {
								v_box.remove(&widget);
							};
							Some(())
						});
				}

				let all_scale = make_channel_scaler("all".to_string(), sound_tx.clone());
				v_box.pack_start(&all_scale, false, true, 0);

				for name in name {
					let channel_scale = make_channel_scaler(name, sound_tx.clone());
					v_box.pack_start(&channel_scale, false, true, 0);
				}

			},
			UIMessage::ChannelChangeSong(_,_) => {

			},
		}
		glib::Continue(true)
	});
	
	window.show_all();
}

fn make_menu_item(
	mnemoic: &str,
	accel_group: &gtk::AccelGroup,
	modifier: gdk::ModifierType,
	keys: &[char]) -> gtk::MenuItem {
	let item = gtk::MenuItem::new_with_mnemonic(mnemoic);
	for key in keys {
		item.add_accelerator("activate", accel_group, *key as u32, modifier, gtk::AccelFlags::VISIBLE);
	}
	item
}

fn make_gamelog_dialog() -> gtk::FileChooserDialog {
	gtk::FileChooserDialog::with_buttons::<gtk::Window>(
		Some("Open gamelog.txt"),
		None,
		gtk::FileChooserAction::Open,
		&[("_Cancel", gtk::ResponseType::Cancel), ("_Open", gtk::ResponseType::Accept)]
	)
}

fn make_soundpack_dialog() -> gtk::FileChooserDialog {
	gtk::FileChooserDialog::with_buttons::<gtk::Window>(
		Some("Open soundpack folder"),
		None,
		gtk::FileChooserAction::SelectFolder,
		&[("_Cancel", gtk::ResponseType::Cancel), ("_Open", gtk::ResponseType::Accept)]
	)
}

fn make_channel_scaler(
	name: String,
	sound_tx: std::sync::mpsc::Sender<SoundMessage>
) -> gtk::Box {
	let box0 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
	let channel_name = gtk::LabelBuilder::new()
		.label(&name)
		.width_request(50)
		.build();
	let scaler = gtk::Scale::new_with_range(
		gtk::Orientation::Horizontal,
		0.0,
		100.0,
		0.01,
	);
	scaler.set_value(100.0);
	scaler.connect_change_value(move |_,_,value| {
		sound_tx.send(
			SoundMessage::VolumeChange(name.clone(), (value/100.0) as f32)
		).unwrap();
		gtk::Inhibit(false)
	});
	box0.pack_start(&channel_name, false, false, 0);
	box0.pack_start(&scaler, true, true, 0);
	box0.show_all();
	box0
}