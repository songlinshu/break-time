pub mod builder;
pub mod prelude;

mod css;
mod state;

use glib::clone;
use gtk::Inhibit;
use std::time::{Duration, Instant};

use prelude::*;
use state::{Message, State};

fn handle_msg_recv(state: &State, msg: Message) {
    // enable(state);

    match msg {
        Message::Display => (),
    }
}

fn end_break(state: &State) {
    for window in state.get_app_wins() {
        window.hide();
    }
}

fn decrement_presses_remaining(state: &State) {
    let remaining = state.decrement_presses_remaining();

    if remaining == 0 {
        end_break(state);
    }
}

fn setup(state: &State) {
    for window in state.get_app_wins() {
        css::setup(window.upcast_ref());
    }
}

fn connect_events(state: &State) {
    for window in state.get_app_wins() {
        window.connect_key_press_event(
            clone!(@strong state => move |_, event_key| {
                if event_key.get_keyval() == gdk::enums::key::space {
                    decrement_presses_remaining(&state);
                    redisplay(&state);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }),
        );
    }

    gtk::timeout_add(
        200,
        clone!(@strong state => move || {

            let now = Instant::now();
            let time_diff = now.saturating_duration_since(state.start_instant);

            // the full time we want to wait for
            let full_time = Duration::new(20, 0);

            let option_time_remaining = full_time.checked_sub(time_diff);

            match option_time_remaining {
                None => {
                    end_break(&state);
                    glib::source::Continue(false)
                }
                Some(time_remaining) => {
                    for label in state.get_time_remaining_labels() {
                        label.set_text(&format!("{:?}", time_remaining));
                    }
                    glib::source::Continue(true)
                }
            }

        }),
    );
}

fn redisplay(state: &State) {
    let presses_remaining = state.read_presses_remaining();

    for label in state.get_presses_remaining_labels() {
        label.set_text(&format!("{}", presses_remaining));
    }
}

pub fn start_break() {
    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let state = State::new(sender);

    setup(&state);

    connect_events(&state);

    redisplay(&state);

    for (window, monitor) in state.get_app_wins_with_monitors() {
        window.show_all();

        let monitor_rect = monitor.get_geometry();
        let gdk_window: gdk::Window = window.get_window().expect(
            "Gtk::Window should always be able to be converted to Gdk::Window",
        );
        gdk_window.fullscreen_on_monitor(monitor.id);
        gdk_window.resize(monitor_rect.width, monitor_rect.height);
    }

    receiver.attach(
        None,
        clone!(@strong state => move |msg| {
            handle_msg_recv(&state, msg);
            glib::source::Continue(true)
        }),
    );
}
