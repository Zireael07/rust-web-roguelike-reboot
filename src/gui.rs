extern crate rltk;
use rltk::{ RGB, Rltk, Console, VirtualKeyCode, Point };
use super::{ Player, Pools, gamelog::GameLog, camera, RunState,
    State, Entity, Name, InBackpack, Equipped, Viewshed, Attributes, Attribute};
extern crate specs;
use specs::prelude::*;

//main menu
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame,
     //LoadGame, 
     Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Neon Twilight");
    
    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        // if selection == MainMenuSelection::LoadGame {
        //     ctx.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
        // } else {
        //     ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
        // }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        // New: Handle web buttons
        if let Some(btn) = &ctx.web_button {
            match btn.trim() {
                "go_n" => {
                    let newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        //MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                    }
                    return MainMenuResult::NoSelection{ selected: newselection }
                }
                "go_s" => {
                    let newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        //MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                    }
                    return MainMenuResult::NoSelection{ selected: newselection }
                }
                "confirm" => return MainMenuResult::Selected{ selected : selection },
                _ => return MainMenuResult::NoSelection{ selected: selection }
            }
        }
        else {
            match ctx.key {
                None => return MainMenuResult::NoSelection{ selected: selection },
                Some(key) => {
                    match key {
                        VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                        VirtualKeyCode::Up => {
                            let newselection;
                            match selection {
                                MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                                //MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                                MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                            }
                            return MainMenuResult::NoSelection{ selected: newselection }
                        }
                        VirtualKeyCode::Down => {
                            let newselection;
                            match selection {
                                MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                                //MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                                MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                            }
                            return MainMenuResult::NoSelection{ selected: newselection }
                        }
                        VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                        _ => return MainMenuResult::NoSelection{ selected: selection }
                    }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

pub fn game_over(ctx : &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Your journey has ended!");
    ctx.print_color_centered(17, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "One day, we'll tell you all about how you did.");
    ctx.print_color_centered(18, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "That day, sadly, is not in this chapter..");

    ctx.print_color_centered(20, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Press any key to return to the menu.");

    // New: Handle web buttons
    if let Some(btn) = &ctx.web_button {
        match btn.trim() {
            _ => GameOverResult::NoSelection,
            "go_n" => GameOverResult::QuitToMenu,
        }
    }
    else {
        match ctx.key {
            None => GameOverResult::NoSelection,
            Some(_) => GameOverResult::QuitToMenu
        }
    }
}


//helper
pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

fn draw_attribute(name : &str, attribute : &Attribute, y : i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray : RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_gray, black, name);
    let color : RGB =
        if attribute.modifiers < 0 { RGB::from_f32(1.0, 0.0, 0.0) }
        else if attribute.modifiers == 0 { RGB::named(rltk::WHITE) }
        else { RGB::from_f32(0.0, 1.0, 0.0) };
    ctx.print_color(67, y, color, black, &format!("{}", attribute.base + attribute.modifiers));
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 { ctx.set(72, y, color, black, rltk::to_cp437('+')); }
}

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    use rltk::to_cp437;
    let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);

    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray, black); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray, black); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray, black); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 11, box_gray, black); // Top-right panel
    
    // Draw box connectors
    ctx.set(0, 45, box_gray, black, to_cp437('├'));
    ctx.set(49, 11, box_gray, black, to_cp437('├'));
    ctx.set(49, 0, box_gray, black, to_cp437('┬'));
    ctx.set(49, 45, box_gray, black, to_cp437('┴'));
    ctx.set(79, 8, box_gray, black, to_cp437('┤'));
    ctx.set(79, 45, box_gray, black, to_cp437('┤'));

    //draw health bar
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let players = ecs.read_storage::<Player>();
    for (_player, pool) in (&players, &pools).join() {
        let health = format!(" HP: {} / {} ", pool.hit_points.current, pool.hit_points.max);
        ctx.print_color(50, 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(64, 1, 14, pool.hit_points.current, pool.hit_points.max, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    //draw attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("STR:", &attr.strength, 4, ctx);
    draw_attribute("DEX:", &attr.dexterity, 5, ctx);
    draw_attribute("CON:", &attr.constitution, 6, ctx);
    draw_attribute("INT:", &attr.intelligence, 7, ctx);
    draw_attribute("WIS:", &attr.wisdom, 8, ctx);
    draw_attribute("CHA:", &attr.charisma, 9, ctx);

    //basic info
    //let player_entity = ecs.fetch::<Entity>();
    let player_pos = ecs.fetch::<Point>();
    //let viewsheds = ecs.read_storage::<Viewshed>();

    let pos = format!("Player: {:?} ", *player_pos);
    ctx.print_color(50, 12, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &pos);

    // let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(ecs, ctx);
    // let x_str = format!("X: {:?}-{:?}", min_x, max_x);
    // let y_str = format!("Y: {:?}-{:?}", min_y, max_y);
    // ctx.print_color(50, 11, RGB::named(rltk::LIGHT_BLUE), RGB::named(rltk::BLACK), &x_str);
    // ctx.print_color(50, 12, RGB::named(rltk::LIGHT_BLUE), RGB::named(rltk::BLACK), &y_str);

    //log
    let log = ecs.fetch::<GameLog>();

    let mut y = 46;
    //slice to get last x
    if log.entries.len() > 13 {
        let slice = log.entries[log.entries.len()-13 .. log.entries.len()].to_vec();
        for s in slice.iter() {
            if y < 59 { ctx.print(2, y, &s.to_string()); }
            y += 1;
        }
    }
    else {
        for s in log.entries.iter() {
            if y < 59 { ctx.print(2, y, &s.to_string()); }
            y += 1;
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

//menus
pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult,  Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    let mut x = 15;
    ctx.draw_box(x, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(x+3, y-2, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(x+3, y+count as i32+1, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    //item.1. is backpack
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        //draw item letter
        ctx.set(x+2, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(x+3, y, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), 97+j as u8);
        ctx.set(x+4, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(x+6, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }


    // New: Handle web buttons
    if let Some(btn) = &ctx.web_button {
        match btn.trim() {
            "escape" => { (ItemMenuResult::Cancel, None) },
            "a" => {
                //select and return
                let selection = 0;
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            },
            "b" => {
                //select and return
                let selection = 1;
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            },
            "c" => {
                //select and return
                let selection = 2;
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            },
            "d" => {
                //select and return
                let selection = 3;
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            },
            _ => (ItemMenuResult::NoResponse, None),
        }
    }
    else {
        match ctx.key {
            None => (ItemMenuResult::NoResponse, None),
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                    _ => {
                        //select item and return it
                        let selection = rltk::letter_to_option(key);
                        if selection > -1 && selection < count as i32 {
                            return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                        }
                        (ItemMenuResult::NoResponse, None)
                    }
                }
            }
        }
    }

}

pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Drop Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), 97+j as u8);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn remove_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Remove Which Item?");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), 97+j as u8);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

//targeting
pub fn ranged_target(gs : &mut State, ctx : &mut Rltk, range : i32) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 1, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1 && screen_x < (max_x - min_x)-1 && screen_y > 1 && screen_y < (max_y - min_y)-1 {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    let mut valid_target = false;
    for idx in available_cells.iter() { if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 { valid_target = true; } }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}