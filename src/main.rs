use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_std::stream::StreamExt;
use serde::Serialize;
use zbus::{dbus_proxy, Connection};

#[derive(Serialize)]
#[serde(tag = "type")]
enum State {
    Stopped,
    Pomodoro,
    ShortBreak,
    LongBreak,
}

impl State {
    fn as_str(&self) -> &str {
        match self {
            Self::Stopped => "stopped",
            Self::Pomodoro => "pomodoro",
            Self::ShortBreak => "short-break",
            Self::LongBreak => "long-break",
        }
    }

    fn from_str(s: &str) -> Result<Self, &str> {
        match s {
            // NOTE: The state is "null" when GNOME Pomodoro is not running.
            "null" => Ok(Self::Stopped),

            "stopped" => Ok(Self::Stopped),
            "pomodoro" => Ok(Self::Pomodoro),
            "short-break" => Ok(Self::ShortBreak),
            "long-break" => Ok(Self::LongBreak),
            _ => Err("invalid state"),
        }
    }
}

struct Properties {
    elapsed: Duration,
    is_paused: bool,
    state: State,
    state_duration: Duration,
}

#[derive(Serialize)]
struct PropertiesJson<'a> {
    remaining_time_in_secs: u64,
    is_paused: bool,
    state: &'a str,
}

#[dbus_proxy(
    interface = "org.gnome.Pomodoro",
    default_service = "org.gnome.Pomodoro",
    default_path = "/org/gnome/Pomodoro"
)]
trait GnomePomodoro {
    #[dbus_proxy(property)]
    fn elapsed(&self) -> zbus::Result<f64>;

    #[dbus_proxy(property)]
    fn is_paused(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property)]
    fn state(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn state_duration(&self) -> zbus::Result<f64>;
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let proxy = GnomePomodoroProxy::new(&connection).await?;

    let (initial_elapsed, initial_is_paused, initial_state, initial_state_duration) = futures_util::try_join!(
        proxy.elapsed(),
        proxy.is_paused(),
        proxy.state(),
        proxy.state_duration()
    )?;
    let props = Arc::new(Mutex::new(Properties {
        elapsed: Duration::from_secs_f64(initial_elapsed),
        is_paused: initial_is_paused,
        state: State::from_str(&initial_state)?,
        state_duration: Duration::from_secs_f64(initial_state_duration),
    }));

    let print_props_json = |props: &Properties| {
        println!(
            "{}",
            serde_json::to_string(&PropertiesJson {
                remaining_time_in_secs: (props.state_duration - props.elapsed).as_secs(),
                is_paused: props.is_paused,
                state: props.state.as_str(),
            })
            .unwrap()
        )
    };

    futures_util::try_join!(
        async {
            let mut elapsed_changed = proxy.receive_elapsed_changed().await;

            while let Some(elapsed) = elapsed_changed.next().await {
                let mut props = props.lock().unwrap();

                props.elapsed = Duration::from_secs_f64(elapsed.get().await?);
                print_props_json(&props);
            }

            Ok::<(), Box<dyn Error>>(())
        },
        async {
            let mut is_paused_changed = proxy.receive_is_paused_changed().await;

            while let Some(is_paused) = is_paused_changed.next().await {
                let mut props = props.lock().unwrap();

                props.is_paused = is_paused.get().await?;
                print_props_json(&props);
            }

            Ok::<(), Box<dyn Error>>(())
        },
        async {
            let mut state_changed = proxy.receive_state_changed().await;

            while let Some(state) = state_changed.next().await {
                let mut props = props.lock().unwrap();

                props.state = State::from_str(&state.get().await?)?;
                print_props_json(&props);
            }

            Ok::<(), Box<dyn Error>>(())
        },
        async {
            let mut state_duration_changed = proxy.receive_state_duration_changed().await;

            while let Some(state_duration) = state_duration_changed.next().await {
                let mut props = props.lock().unwrap();

                props.state_duration = Duration::from_secs_f64(state_duration.get().await?);
                print_props_json(&props);
            }

            Ok::<(), Box<dyn Error>>(())
        },
    )?;

    Ok(())
}
