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
            error!(%err.error, "{}", err.title);
            main_window
                .invoke_show_error_popup((&err.title).into(), (&err.error).to_string().into());
            return Err(err);
        }
    }
}
