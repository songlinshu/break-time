#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod opts;
mod prelude;
mod scheduler;
mod tray;
pub mod ui;
mod x11;

use std::sync::mpsc::Sender;
use std::time::Duration;

use config::Config;
use prelude::*;
use scheduler::Scheduler;
use tray::Tray;

pub enum Msg {
    EndBreak,
    Pause,
    Quit,
    ResetSysTrayIcon,
    Resume,
    StartBreak,
    TimeRemainingBeforeBreak(Duration),
}

fn handle_msg_recv(
    config: &Config,
    sender: glib::Sender<Msg>,
    scheduler_outer_sender: Sender<scheduler::Msg>,
    scheduler_inner_sender: Sender<scheduler::InnerMsg>,
    tray: &mut Tray,
    msg: Msg,
) {
    match msg {
        Msg::EndBreak => {
            println!("break ended");
            scheduler_outer_sender.send(scheduler::Msg::Start);
        }
        Msg::Pause => {
            tray.pause();
            scheduler_inner_sender.send(scheduler::InnerMsg::Pause);
        }
        Msg::Quit => {
            gtk::main_quit();
        }
        Msg::StartBreak => {
            println!("starting break");
            tray.render_break_starting();
            ui::start_break(config, sender);
        }
        Msg::ResetSysTrayIcon => {
            tray.render_normal_icon();
        }
        Msg::Resume => {
            tray.resume();
            scheduler_outer_sender.send(scheduler::Msg::Start);
        }
        Msg::TimeRemainingBeforeBreak(remaining_time) => {
            tray.update_time_remaining(remaining_time);
        }
    }
}

pub fn default_main() {
    let opts = opts::Opts::parse_from_args();

    let config = Config::load(opts).expect("Could not load config file.");

    gtk::init().expect("Could not initialize GTK");

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let mut tray = tray::Tray::run(sender.clone());

    println!("Starting the scheduler...");
    let (scheduler_outer_sender, scheduler_inner_sender) =
        Scheduler::run(&config, sender.clone());

    receiver.attach(None, move |msg| {
        handle_msg_recv(
            &config,
            sender.clone(),
            scheduler_outer_sender.clone(),
            scheduler_inner_sender.clone(),
            &mut tray,
            msg,
        );
        glib::source::Continue(true)
    });

    gtk::main();
}
