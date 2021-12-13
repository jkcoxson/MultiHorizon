// jkcoxson

use std::{
    fs::{self},
    io::{self},
    path::Path,
    process::Command,
};

use cursive::{
    align::HAlign,
    event::EventResult,
    theme,
    traits::{Boxable, Nameable, Scrollable},
    views::{Dialog, EditView, OnEventView, SelectView},
    Cursive, CursiveExt, With,
};

// Path to documents folder
const SAVE_PATH: &str = "MultiHorizon";
const GAME_PATH: &str = "Horizon Zero Dawn";

fn main() {
    let mut siv = Cursive::default();

    // Chang color scheme to hacker theme
    let theme = siv.current_theme().clone().with(|theme| {
        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Black);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::TitlePrimary] =
            theme::Color::Light(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::Background] =
            theme::Color::Dark(theme::BaseColor::Black);
        theme.palette[theme::PaletteColor::Secondary] =
            theme::Color::Light(theme::BaseColor::Green);
    });
    siv.set_theme(theme);

    // Get home directory
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.join("Documents");
    let game_dir = home_dir.join(GAME_PATH);

    // Check if the MultiHorizon folder exists
    let save_path = home_dir.join(SAVE_PATH);
    if !save_path.exists() {
        fs::create_dir_all(&save_path).expect("Failed to create save path");
    }

    // Preflight checks of the current system
    if game_dir.is_symlink() {
        // Remove the symlink
        fs::remove_dir(&game_dir).expect("Failed to remove symlink");
    } else {
        // Determine if it even exists
        if game_dir.exists() {
            // User is installed incorrectly
            // Get new username
            let username = text_prompt(
                &mut siv,
                "User is installed incorrectly. Please enter your username.",
            );
            // Create new path in save folder
            let save_dir = save_path.join(username);
            if !save_dir.exists() {
                fs::create_dir_all(&save_dir).expect("Failed to create save path");
            } else {
                siv.add_layer(
                    cursive::views::Dialog::text(
                        "A user with that username already exists.\n\nPlease try again."
                            .to_string(),
                    )
                    .button("Ok", |s| s.quit()),
                );
                siv.run();
                return;
            }
            // Move game_dir to save_dir
            recursive_move(&game_dir, &save_dir).unwrap();
        } else {
            // We gucci because nothing is installed
        }
    }

    // Get list of users in save_dir
    let mut users = Vec::new();
    for entry in fs::read_dir(&save_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            users.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    users.push("New User".to_string());

    // Get selected user
    let mut selected_user = select_prompt(&mut siv, "Choose a user to load", users);

    if selected_user == "New User".to_string() {
        // Create new folder in save_dir
        let new_user = text_prompt(&mut siv, "Enter a username");
        if new_user == "New User".to_string() {
            siv.add_layer(
                cursive::views::Dialog::text("lol".to_string()).button("Ok", |s| s.quit()),
            );
            siv.run();
            return;
        }
        let save_dir = save_path.join(&new_user);
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir).expect("Failed to create save path");
        } else {
            siv.add_layer(
                cursive::views::Dialog::text(
                    "A user with that username already exists.\n\nPlease try again.".to_string(),
                )
                .button("Ok", |s| s.quit()),
            );
            siv.run();
            return;
        }
        selected_user = new_user;
    }

    // Create symlink from save_dir/selected_user/Horizon Zero Dawn to home_dir/Horizon Zero Dawn
    let save_dir = save_path.join(selected_user);
    let game_dir = home_dir.join(GAME_PATH);
    create_symlink(&save_dir, &game_dir).unwrap();
    open::that("steam://rungameid/1151640").unwrap();
}

fn text_prompt(siv: &mut Cursive, title: &str) -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    let cloned_tx = tx.clone();

    siv.add_layer(
        Dialog::new()
            .title(title)
            // Padding is (left, right, top, bottom)
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    // Call `show_popup` when the user presses `Enter`
                    .on_submit(move |s, name| {
                        s.pop_layer();
                        s.quit();
                        cloned_tx.send(name.to_string()).unwrap();
                    })
                    // Give the `EditView` a name so we can refer to it later.
                    .with_name("name")
                    // Wrap this in a `ResizedView` with a fixed width.
                    // Do this _after_ `with_name` or the name will point to the
                    // `ResizedView` instead of `EditView`!
                    .fixed_width(20),
            )
            .button("Ok", move |s| {
                // This will run the given closure, *ONLY* if a view with the
                // correct type and the given name is found.
                let name = s
                    .call_on_name("name", |view: &mut EditView| {
                        // We can return content from the closure!
                        view.get_content()
                    })
                    .unwrap();

                // Run the next step
                s.pop_layer();
                s.quit();
                tx.send(name.to_string()).unwrap();
            }),
    );
    siv.run();
    rx.recv().unwrap().replace("\"", "")
}

fn select_prompt(siv: &mut Cursive, title: &str, options: Vec<String>) -> String {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump()
        .on_submit(move |s, choice: &str| {
            let choice = choice.to_string();
            tx.send(choice).unwrap();
            s.quit();
        });
    select.add_all_str(options);

    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        });

    siv.add_layer(Dialog::around(select.scrollable().fixed_size((20, 10))).title(title));
    siv.run();
    rx.recv().unwrap()
}

fn recursive_move(src: &Path, dest: &Path) -> io::Result<()> {
    for entry in src.read_dir()? {
        let entry = entry?;
        let src = entry.path();
        let dest = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir(&dest)?;
            recursive_move(&src, &dest)?;
        } else {
            fs::copy(&src, &dest)?;
        }
    }
    recursive_remove(src)?;
    Ok(())
}

fn recursive_remove(src: &Path) -> io::Result<()> {
    if src.is_dir() {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src = entry.path();
            recursive_remove(&src)?;
        }
        fs::remove_dir(src)?;
    } else {
        fs::remove_file(src)?;
    }

    Ok(())
}

fn create_symlink(src: &Path, dest: &Path) -> io::Result<()> {
    // Run the mklink command
    Command::new("cmd")
        .args(&[
            "/C",
            "mklink",
            "/J",
            dest.to_str().unwrap(),
            src.to_str().unwrap(),
        ])
        .spawn()?;
    Ok(())
}
