use std::fmt::{Debug, Display};

use tracing::error;

use crate::MainWindow;

pub struct Error {
    pub title: String,
    pub error: Box<dyn Display>,
}
impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("title", &self.title)
            .field("error", &self.error.to_string())
            .finish()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title, self.error)
    }
}
impl std::error::Error for Error {}

#[macro_export]
macro_rules! my_error {
    ($title: expr, $error: expr) => {
        crate::error::Error {
            title: $title.to_string(),
            error: Box::new($error),
        }
    };
    ($title: expr, $error: expr,) => {
        crate::error::Error {
            title: $title.to_string(),
            error: Box::new($error),
        }
    };
}

pub fn wrap_errorable_function<T>(
    main_window: &MainWindow,
    mut f: impl FnMut() -> Result<T, Error>,
) -> Result<T, Error> {
    match f() {
        Ok(t) => Ok(t),
        Err(err) => {
            error!("{}: {}", err.title, err.error);
            main_window.invoke_show_error_popup((&err.title).into(), (&err.error).to_string().into());
            return Err(err);
        }
    }
}
// pub fn wrap_errorable_function_fe<T>(
//     window: &FleetEditorWindow,
//     mut f: impl FnMut() -> Result<T, Error>,
// ) -> Result<T, Error> {
//     match f() {
//         Ok(t) => Ok(t),
//         Err(err) => {
//             error!(%err.error, "{}", err.title);
//             window.invoke_show_error_popup((&err.title).into(), (&err.error).to_string().into());
//             return Err(err);
//         }
//     }
// }
// pub fn wrap_errorable_function_m<T>(
//     window: &MissileWindow,
//     mut f: impl FnMut() -> Result<T, Error>,
// ) -> Result<T, Error> {
//     match f() {
//         Ok(t) => Ok(t),
//         Err(err) => {
//             error!("{}: {}", err.title, err.error);
//             window.invoke_show_error_popup((&err.title).into(), (&err.error).to_string().into());
//             return Err(err);
//         }
//     }
// }
