use iced::{
    alignment::Horizontal,
    executor,
    keyboard::{key::Named, on_key_press, Key},
    widget::{button, center, column, row, text},
    Task, Theme,
};
use iced_fonts::{nerd::icon_to_string, Nerd, NERD_FONT};
use iced_layershell::{to_layer_message, Application};
use logind_zbus::{manager::ManagerProxy, session::SessionProxy};
use miette::Diagnostic;
use thiserror::Error;
use zbus::{connection, Connection};

use crate::config::Config;

pub struct App {
    config: Config,
    connection: Option<Connection>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Task<Self::Message>) {
        (
            Self {
                config,
                connection: None,
            },
            Task::perform(Self::zbus_connect(), Self::Message::ZbusConnected),
        )
    }

    fn namespace(&self) -> String {
        "com.tukanoidd.waypwr".into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        on_key_press(|key, _| match key {
            Key::Named(named) => match named {
                Named::Escape => Some(Self::Message::Quit),
                _ => None,
            },
            Key::Character(c) => match c.as_str() {
                "q" => Some(Self::Message::Quit),
                _ => None,
            },
            Key::Unidentified => None,
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Self::Message::Quit => iced::exit(),

            Self::Message::ZbusConnected(connection) => match connection {
                Ok(connection) => {
                    self.connection = Some(connection);
                    Task::none()
                }
                Err(e) => {
                    tracing::error!("{}", e);
                    Task::done(Self::Message::Quit)
                }
            },

            Self::Message::Lock => Task::perform(
                Self::lock(self.connection.clone()),
                Self::Message::LogindResult,
            ),
            Self::Message::LogOut => Task::perform(
                Self::terminate(self.connection.clone()),
                Self::Message::LogindResult,
            ),
            Self::Message::Hibernate => Task::perform(
                Self::hibernate(self.connection.clone()),
                Self::Message::LogindResult,
            ),
            Self::Message::Reboot => Task::perform(
                Self::reboot(self.connection.clone()),
                Self::Message::LogindResult,
            ),
            Self::Message::Shutdown => Task::perform(
                Self::power_off(self.connection.clone()),
                Self::Message::LogindResult,
            ),

            Self::Message::LogindResult(result) => match result {
                Ok(_) => Task::none(),
                Err(err) => {
                    tracing::error!("Failed to perform logind action: {err}");
                    iced::exit()
                }
            },

            _ => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        // Lock,
        // LogOut,
        // Hibernate,
        // Reboot,
        // Shutdown,
        let waypwr_btn = |icon: Nerd, str: &'static str, msg: Self::Message| {
            button(center(
                column![
                    text(icon_to_string(icon)).font(NERD_FONT).size(80),
                    text(str).size(30)
                ]
                .align_x(Horizontal::Center)
                .spacing(10),
            ))
            .width(200)
            .height(200)
            .on_press(msg)
        };

        center(
            row![
                waypwr_btn(Nerd::AccountLock, "Lock", Self::Message::Lock),
                waypwr_btn(Nerd::Logout, "Log Out", Self::Message::LogOut),
                waypwr_btn(Nerd::Snowflake, "Hibernate", Self::Message::Hibernate),
                waypwr_btn(Nerd::RotateLeft, "Reboot", Self::Message::Reboot),
                waypwr_btn(Nerd::Power, "Shutdown", Self::Message::Shutdown)
            ]
            .spacing(20),
        )
        .into()
    }

    fn theme(&self) -> Self::Theme {
        self.config.theme.clone()
    }
}

macro_rules! login_fns {
    (
        $(
            $root:ident => [
                $(
                    $fn:ident [$context:literal] $((
                        $($param:expr),+
                        $(,)?
                    ))?
                ),+
                $(,)?
            ]
        ),+
        $(,)?
    ) => {
        $($(
            async fn $fn(connection: Option<Connection>) -> AppResult<()> {
                let Some(connection) = connection else {
                    return Err(AppError::NoDBusConnection);
                };

                Ok(Self::$root(&connection)
                    .await?
                    .$fn($($($param),+)?)
                    .await?)
            }
        )+)+
    }
}

impl App {
    async fn zbus_connect() -> AppResult<Connection> {
        Ok(connection::Builder::system()?
            .internal_executor(false)
            .build()
            .await?)
    }

    async fn get_logind_manager(connection: &'_ Connection) -> AppResult<ManagerProxy<'_>> {
        Ok(ManagerProxy::new(connection).await?)
    }

    async fn get_logind_session(connection: &'_ Connection) -> AppResult<SessionProxy<'_>> {
        Ok(SessionProxy::new(connection).await?)
    }

    login_fns![
        get_logind_session => [
            lock["Failed to lock the session"],
            terminate["Failed to terminate the session"],
        ],
        get_logind_manager => [
            hibernate["Failed to hibernate"](false),
            reboot["Failed to reboot"](false),
            power_off["Failed to power off"](false),
        ],
    ];
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum AppMsg {
    Quit,

    ZbusConnected(AppResult<Connection>),

    Lock,
    LogOut,
    Hibernate,
    Reboot,
    Shutdown,

    LogindResult(AppResult<()>),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum AppError {
    #[error("No dbus connection!")]
    NoDBusConnection,

    #[error("Failed to connect to session bus: {0}")]
    ZBus(#[from] zbus::Error),
}
