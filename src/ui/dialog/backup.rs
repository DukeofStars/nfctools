use std::{fs::{File, OpenOptions}, path::{Path, PathBuf}};

use dioxus::{prelude::*};
use dioxus_primitives::checkbox::CheckboxState;
use rfd::AsyncFileDialog;
use color_eyre::Result;
use zip::write::FileOptions;

use crate::{components::checkbox::Checkbox, config::APP_CONFIG, ui::dialog::DialogWrapper};

#[component]
pub fn BackupDialog(signal: Signal<bool>) -> Element {
    let mut compress = use_signal(|| true);
    let mut running = use_signal(|| false);

    let mut show_warning = use_signal(|| false);
    let mut warning_path = use_signal(String::new);

    rsx! {
        DialogWrapper { signal: show_warning,
            if show_warning() {
                div { style: "display: flex; flex-direction: column;",
                    h2 { "Warning" }
                    "This action will replace all files in this directory: {warning_path}"
                    div { style: "
                    display: flex;
                    flex-direction: row;
                    justify-content: center;
                    gap: 10px;
                    margin-top: 10px;",
                        button {
                            style: "width: 30%; height: 30px;",
                            class: "button",
                            onclick: move |_| {
                                running.set(false);
                                show_warning.set(false);
                            },
                            "Cancel"
                        }
                        button {
                            style: "width: 30%; height: 30px;",
                            class: "button",
                            onclick: move |_| {
                                let path = PathBuf::from(warning_path());
                                show_warning.set(false);
                                let _ = backup_to_folder(path.as_path());
                                running.set(false);
                                signal.set(false);
                            },
                            "Ok"
                        }
                    }
                }
            } else {

            }
        }
        div { style: "display: flex; flex-direction: column; width: 100%; gap: 10px;",
            h2 { style: "margin: 0px auto 0px", "Backup Fleets" }
            div { style: "margin: 0px auto 0px; display: flex; flex-direction: row; justify-content: center; gap: 5px",
                "Use Compression"
                Checkbox {
                    checked: if compress() { CheckboxState::Checked } else { CheckboxState::Unchecked },
                    on_checked_change: move |checked| {
                        match checked {
                            CheckboxState::Checked => compress.set(true),
                            CheckboxState::Indeterminate => {}
                            CheckboxState::Unchecked => compress.set(false),
                        }
                    },
                }
            }
            button {
                disabled: running(),
                style: "margin: 0px auto 0px; height: 30px; width: 60%",
                class: "button",
                onclick: move |_| {
                    spawn(async move {
                        running.set(true);
                        if compress() {
                            let Some(path) = AsyncFileDialog::new()
                                .add_filter("Zip File", &["zip"])
                                .save_file()
                                .await else {
                                warn!("Backup aborted, no path selected");
                                running.set(false);
                                return;
                            };
                            let _ = backup_to_zip(path.path());
                            running.set(false);
                            signal.set(false);
                        } else {
                            let Some(path) = AsyncFileDialog::new().pick_folder().await else {
                                warn!("Backup aborted, no path selected");
                                running.set(false);
                                return;
                            };
                            let path = path.path();
                            if path
                                .read_dir()
                                .map(|read_dir| read_dir.count())
                                .is_ok_and(|count| count > 0)
                            {
                                warning_path.set(path.display().to_string());
                                show_warning.set(true);
                            } else {
                                let _ = backup_to_folder(path);
                                running.set(false);
                                signal.set(false);
                            }
                        }
                    });
                },
                if running() {
                    span { class: "spinner" }
                } else {
                    "Backup Fleets"
                }
            }
        }
    }
}

fn backup_to_zip(out_path: &Path) -> Result<()> {
    let fleets_root = APP_CONFIG.get().unwrap().lock().unwrap().saves_dir.join("Fleets");

    let mut file = OpenOptions::new().write(true).create(true).open(out_path)?;
    let mut zip_writer = zip::ZipWriter::new_stream(&mut file);

    let mut dirs_queue = Vec::new();

    dirs_queue.push(fleets_root.clone());

    while !dirs_queue.is_empty() {
        let dir = dirs_queue.remove(0);
        let read_dir = dir.read_dir()?;
        for child in read_dir {
            let Ok(entry) = child else { continue };
            let path = entry.path();

            let sub_path = path.strip_prefix(&fleets_root).unwrap();

            if path.is_dir() {
                zip_writer.add_directory_from_path(sub_path, FileOptions::DEFAULT)?;
                dirs_queue.push(path);
            } else if path.is_file() && path.extension().is_some_and(|ext| ext.to_str().unwrap() == "fleet") {
                zip_writer.start_file_from_path(sub_path, FileOptions::DEFAULT)?;
                let mut src_file = File::open(&path)?;
                std::io::copy(&mut src_file, &mut zip_writer)?;
            }
        }
    }

    zip_writer.finish()?;

    Ok(())
}

fn backup_to_folder(out_path: &Path) -> Result<()> {
    let fleets_root = APP_CONFIG.get().unwrap().lock().unwrap().saves_dir.join("Fleets");

    let mut dirs_queue = Vec::new();

    dirs_queue.push(fleets_root.clone());

    while !dirs_queue.is_empty() {
        let dir = dirs_queue.remove(0);
        let read_dir = dir.read_dir()?;
        for child in read_dir {
            let Ok(entry) = child else {continue};
            let path = entry.path();

            let sub_path = path.strip_prefix(&fleets_root).unwrap();

            let new_path = out_path.join(sub_path);
            if path.is_dir() {
                dirs_queue.push(path);
                if !new_path.exists() {
                    trace!("Creating directory: '{}'", new_path.display());
                    std::fs::create_dir(new_path)?;
                }
            } else if path.is_file() && path.extension().is_some_and(|ext| ext.to_str().unwrap() == "fleet") {
                if new_path.exists() {
                    trace!("Removing old file: '{}'", new_path.display());
                    std::fs::remove_file(&new_path)?;
                }
                trace!("Copying file: '{}' -> '{}'", path.display(), new_path.display());
                std::fs::copy(&path, &new_path)?;
            }
        }
    }

    Ok(())
}