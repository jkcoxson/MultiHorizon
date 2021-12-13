// jkcoxson

use std::{
    fs,
    io::{self, Write},
    path::Path,
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

    // Check if Horizon Zero Dawn folder exists
    let game_dir = home_dir.join(GAME_PATH.clone());
    if !game_dir.exists() {
        siv.add_layer(
            cursive::views::Dialog::text("Horizon Zero Dawn folder not found.\n\nPlease make sure you have Horizon Zero Dawn installed and try again.".to_string())
                .button("Ok", |s| s.quit()),
        );
        siv.run();
        return;
    }

    // Check if the MultiHorizon folder exists
    let save_dir = home_dir.join(SAVE_PATH);
    if !save_dir.exists() {
        // Create directory
        std::fs::create_dir_all(&save_dir).unwrap();
    }

    // Determine what user is loaded
    // Find a file in the game folder ending in .mhzd
    let mut loaded_user: Option<String> = None;
    for entry in fs::read_dir(&game_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = match path.extension() {
            Some(extension) => extension,
            None => continue,
        };
        if extension == "mhzd" {
            loaded_user = Some(
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .split('.')
                    .next()
                    .unwrap()
                    .to_string(),
            );

            break;
        }
    }

    // If no user is detected, ask the user for a username
    if loaded_user.is_none() {
        let username = text_prompt(&mut siv, "No user detected, please enter your username");

        // Check to see if a folder with the same name exists in the save directory
        let save_dir = save_dir.join(username.clone());
        if save_dir.exists() {
            siv.add_layer(
                cursive::views::Dialog::text(
                    "A user with that username already exists.\n\nPlease try again.".to_string(),
                )
                .button("Ok", |s| s.quit()),
            );
            siv.run();
            return;
        }

        // Create user file
        let game_dir = home_dir.join(GAME_PATH);
        let user_file = game_dir.join(format!("{}.mhzd", username));
        let mut file = fs::File::create(&user_file).unwrap();
        file.write_all(b"").unwrap();

        // Create new folder with username in save directory
        std::fs::create_dir(&save_dir).unwrap();

        move_to_save(username);
    }

    // Get list of users in the save dir
    let mut users: Vec<String> = Vec::new();
    for entry in fs::read_dir(&save_dir).unwrap() {
        // If entry is not a directory, skip it
        if !entry.as_ref().unwrap().file_type().unwrap().is_dir() {
            continue;
        }
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        users.push(name);
    }
    users.push("New User".to_string());

    let selected_user = select_prompt(&mut siv, "Select a user", users);
    if selected_user == "New User" {
        // Unload current user
        move_to_save(loaded_user.clone().unwrap());
        // Remove all files in game folder
        for entry in fs::read_dir(game_dir).unwrap() {
            recursive_remove(&entry.unwrap().path()).unwrap();
        }

        let username = text_prompt(&mut siv, "Enter a new username");

        // Check to see if a folder with the same name exists in the save directory
        let save_dir = save_dir.join(username.clone());
        if save_dir.exists() {
            siv.add_layer(
                cursive::views::Dialog::text(
                    "A user with that username already exists.\n\nPlease try again.".to_string(),
                )
                .button("Ok", |s| s.quit()),
            );
            siv.run();
            return;
        }

        // Create user file
        let game_dir = home_dir.join(GAME_PATH);
        let user_file = game_dir.join(format!("{}.mhzd", username));
        let mut file = fs::File::create(&user_file).unwrap();
        file.write_all(b"").unwrap();

        // Create new folder with username in save directory
        std::fs::create_dir(&save_dir).unwrap();
    } else {
        // Load user
        if selected_user != loaded_user.clone().unwrap() {
            // Unload current user
            move_to_save(loaded_user.unwrap());

            // Load new user
            move_to_game(selected_user.clone());
        }
    }

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

fn move_to_save(username: String) {
    let home_dir = dirs::home_dir().unwrap().join("Documents");
    let save_dir = home_dir.join(SAVE_PATH).join(&username);
    let game_dir = home_dir.join(GAME_PATH);

    // Remove all files in the save directory
    for entry in fs::read_dir(&save_dir).unwrap() {
        recursive_remove(&entry.unwrap().path()).unwrap();
    }

    // Copy all files from game folder to save directory
    for entry in fs::read_dir(game_dir).unwrap() {
        match recursive_copy(
            &entry.as_ref().unwrap().path(),
            &save_dir.join(entry.unwrap().file_name()),
        ) {
            Err(e) => panic!("{}", e),
            Ok(_) => {}
        }
    }
}

fn move_to_game(username: String) {
    let home_dir = dirs::home_dir().unwrap().join("Documents");
    let save_dir = home_dir.join(SAVE_PATH).join(&username);
    let game_dir = home_dir.join(GAME_PATH);

    // Remove all files in the game directory
    for entry in fs::read_dir(&game_dir).unwrap() {
        recursive_remove(&entry.unwrap().path()).unwrap();
    }

    // Copy all files from game folder to save directory
    for entry in fs::read_dir(save_dir).unwrap() {
        match recursive_copy(
            &entry.as_ref().unwrap().path(),
            &game_dir.join(entry.unwrap().file_name()),
        ) {
            Err(e) => panic!("{}", e),
            Ok(_) => {}
        }
    }
}

fn recursive_copy(src: &Path, dest: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src = entry.path();
            let dest = dest.join(src.file_name().unwrap());
            recursive_copy(&src, &dest)?;
        }
    } else {
        fs::copy(src, dest)?;
    }

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
