#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use derive_more::{Deref, DerefMut};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};

fn main() {
    dioxus_desktop::launch_cfg(App, dioxus_desktop::Config::new());
}

fn App(cx: Scope) -> Element {
    let first_value = use_state(cx, || String::new());
    let second_value = use_state(cx, || String::new());
    let first = vec!["First", "Second", "Third"];
    let second = vec!["fourth", "fifth", "sixth"];
    let items = use_state(cx, || first.clone());
    render! {
        div {
            role: "application",
            Listbox {
                label: "First",
                value: first_value,
                onmounted: move |event: MountedEvent| {
                    event.inner().set_focus(true);
                },
                onchange: move |new: String| {
                    match new.as_str() {
                        "first" => {
                            items.set(first.clone());
                        }
                        "second" => {
                            items.set(second.clone());
                        }
                        _ => unimplemented!(),
                    };
                },
                ListboxOption {
                    id: "first",
                    "First list"
                }
                ListboxOption {
                    id: "second",
                    "Second list"
                }
            }
            Listbox {
                label: "second",
                value: second_value,
                for item in items.iter() {
                    ListboxOption {
                        id: item,
                        item.to_string()
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Deref, DerefMut)]
struct Items {
    #[deref]
    #[deref_mut]
    items: Vec<Item>,
    selected: Option<ScopeId>,
    active: String,
}

#[derive(Debug)]
struct Item {
    id: String,
    scope_id: ScopeId,
}

#[inline_props]
pub fn Listbox<'a>(
    cx: Scope<'a>,
    label: &'a str,
    value: &'a UseState<String>,
    onmounted: Option<EventHandler<'a, MountedEvent>>,
    onchange: Option<EventHandler<'a, String>>,
    children: Element<'a>,
) -> Element {
    use_shared_state_provider(cx, Items::default);
    let items = use_shared_state::<Items>(cx).unwrap();
    {
        let items_read = items.read();
        if items_read.active.is_empty() && !value.is_empty() {
            drop(items_read);
            let mut items_write = items.write();
            let value = value.get();
            if let Some(onchange) = onchange {
                onchange.call(value.clone());
            }
            items_write.active = value.clone();
        }
    }
    use_shared_state_provider(cx, || 0_usize);
    let mut selected_index = use_state(cx, || 0);
    let update_selection = move || {
        let mut items = items.write();
        let index = *(selected_index.current());
        let active = items.active.clone();
        if items.is_empty() && items.selected.is_some() {
            items.selected = None;
            items.active = String::new();
            value.set(String::new());
        } else if index < items.len() {
            items.selected = Some(items[index].scope_id);
            let id = items[index].id.clone();
            items.active = id.clone();
            value.set(id);
        }
        if !items.active.is_empty() && items.active != active {
            if let Some(on_change) = onchange {
                on_change.call(items.active.clone());
            }
        }
    };
    let should_set_initial_selection = !items.read().is_empty() && value.is_empty();
    if should_set_initial_selection {
        update_selection();
    }
    let active = {
        let items = items.read();
        items.active.clone()
    };
    render! {
        ul {
            role: "listbox",
            class: "list-group",
            aria_label: "{label}",
            aria_activedescendant: "{active}",
            tabindex: 0,
            onmounted: move |event| {
                if let Some(v) = onmounted {
                    v.call(event);
                }
            },
            onkeydown: move |event| {
                match event.key() {
                    Key::ArrowDown => {
                        let items = items.read();
                        if **selected_index < items.len() {
                            selected_index += 1;
                            if *(selected_index.current()) >= items.len() {
                                selected_index.set(items.len().saturating_sub(1));
                            }
                            drop(items);
                            update_selection();
                        }
                    }
                    Key::ArrowUp => {
                        if **selected_index > 0 {
                            selected_index -= 1;
                            update_selection();
                        }
                    }
                    Key::Home => {
                        selected_index.set(0);
                        update_selection();
                    }
                    Key::End => {
                        {
                            let items = items.read();
                            selected_index.set(items.len().saturating_sub(1));
                        }
                        update_selection();
                    }
                    _ => {}
                }
            },
            children
        }
    }
}

struct ItemsEntry {
    id: String,
    items: UseSharedState<Items>,
    scope_id: ScopeId,
}

impl Drop for ItemsEntry {
    fn drop(&mut self) {
        println!("Dropping {}", self.id);
        let mut items = self.items.write();
        items.retain(|v| v.scope_id != self.scope_id);
    }
}

#[inline_props]
pub fn ListboxOption<'a>(cx: Scope<'a>, id: &'a str, children: Element<'a>) -> Element {
    let items = use_shared_state::<Items>(cx).unwrap();
    cx.use_hook(|| {
        {
            let mut items = items.write();
            items.push(Item {
                scope_id: cx.scope_id(),
                id: id.to_string(),
            });
            if id == &items.active {
                items.selected = Some(cx.scope_id());
            }
        }
        ItemsEntry {
            id: id.to_string(),
            items: items.clone(),
            scope_id: cx.scope_id(),
        }
    });
    let selected = items
        .read()
        .selected
        .map(|id| id == cx.scope_id())
        .unwrap_or_else(|| false);
    let class = if selected {
        "list-group-item active"
    } else {
        "list-group-item"
    };
    render! {
        li {
            role: "option",
            class: class,
            aria_selected: selected,
            key: "{id}",
            id: "{id}",
            children
        }
    }
}
