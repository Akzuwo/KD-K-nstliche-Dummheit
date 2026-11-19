use dioxus::prelude::*;
use kd_desktop::{
    engine::{self, Event},
    models::{self, Model},
    store::{self, Chat, Message, Store},
};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

#[derive(Clone)]
struct Services {
    store: Store,
    root: PathBuf,
    tx: mpsc::Sender<Update>,
    cancel: Arc<AtomicBool>,
    working: Arc<AtomicBool>,
}
enum Update {
    Engine(Event),
    ImportStatus(String),
    Imported(Result<Model, String>),
}
#[derive(Clone)]
struct Ui {
    chats: Vec<Chat>,
    messages: Vec<Message>,
    models: Vec<Model>,
    active: String,
    chat: String,
    input: String,
    search: String,
    sidebar: bool,
    model_menu: bool,
    panel: String,
    theme: String,
    backend: String,
    busy: bool,
    importing: bool,
    status: String,
    error: String,
    stream_id: String,
    stream_chat: String,
    stream_text: String,
    rename_id: String,
    rename_title: String,
    delete_kind: String,
    delete_id: String,
}
impl Ui {
    fn load(s: &Services) -> Self {
        let models = models::list(&s.root);
        let mut error = String::new();
        let models = match models {
            Ok(m) => m,
            Err(e) => {
                error = format!("Modelle konnten nicht gelesen werden: {e}");
                vec![]
            }
        };
        let active = s.store.setting("active_model");
        let active = if models.iter().any(|m| m.id == active) {
            active
        } else {
            models.first().map(|m| m.id.clone()).unwrap_or_default()
        };
        let theme = s.store.setting("theme");
        let backend = s.store.setting("backend");
        let chats = s.store.chats("").unwrap_or_else(|e| {
            error = e.to_string();
            vec![]
        });
        Self {
            chats,
            messages: vec![],
            models,
            active,
            chat: String::new(),
            input: String::new(),
            search: String::new(),
            sidebar: true,
            model_menu: false,
            panel: String::new(),
            theme: if theme.is_empty() {
                "system".into()
            } else {
                theme
            },
            backend: if backend == "cuda" {
                "cuda".into()
            } else {
                "cpu".into()
            },
            busy: false,
            importing: false,
            status: String::new(),
            error,
            stream_id: String::new(),
            stream_chat: String::new(),
            stream_text: String::new(),
            rename_id: String::new(),
            rename_title: String::new(),
            delete_kind: String::new(),
            delete_id: String::new(),
        }
    }
    fn refresh(&mut self, s: &Services) -> anyhow::Result<()> {
        self.chats = s.store.chats(&self.search)?;
        if !self.chat.is_empty() {
            self.messages = s.store.messages(&self.chat)?;
        }
        Ok(())
    }
    fn model_label(&self) -> String {
        self.models
            .iter()
            .find(|m| m.id == self.active)
            .map(|m| format!("{} {}", m.name, m.version))
            .unwrap_or("Modell auswählen".into())
    }
}
fn failure(mut ui: Signal<Ui>, e: impl std::fmt::Display) {
    ui.write().error = e.to_string();
}
fn fresh(mut ui: Signal<Ui>) {
    let mut u = ui.write();
    u.chat.clear();
    u.messages.clear();
    u.input.clear();
    u.model_menu = false;
}
fn select_model(s: &Services, mut ui: Signal<Ui>, id: String) {
    if ui.read().busy {
        return;
    }
    match s.store.set_setting("active_model", &id) {
        Ok(_) => {
            let mut u = ui.write();
            u.active = id;
            u.model_menu = false;
        }
        Err(e) => failure(ui, e),
    }
}

fn set_theme(s: &Services, mut ui: Signal<Ui>, theme: &str) {
    match s.store.set_setting("theme", theme) {
        Ok(()) => ui.write().theme = theme.into(),
        Err(e) => failure(ui, e),
    }
}
fn import(s: Services, mut ui: Signal<Ui>) {
    if ui.read().busy || ui.read().importing {
        return;
    }
    ui.write().importing = true;
    ui.write().error.clear();
    std::thread::spawn(move || {
        let file = rfd::FileDialog::new()
            .set_title("KD-Modellpaket importieren")
            .add_filter("KD-Modellpaket", &["zip"])
            .pick_file();
        let result = match file {
            Some(file) => models::import(&s.root, &file, |p| {
                let _ = s.tx.send(Update::ImportStatus(p));
            })
            .map_err(|e| format!("{e:#}")),
            None => Err(String::new()),
        };
        let _ = s.tx.send(Update::Imported(result));
    });
}
fn submit(s: Services, mut ui: Signal<Ui>) {
    let u = ui.read().clone();
    if u.busy || u.importing || u.input.trim().is_empty() {
        return;
    }
    let Some(model) = u.models.iter().find(|m| m.id == u.active).cloned() else {
        ui.write().panel = "models".into();
        return;
    };
    if s.working.swap(true, Ordering::SeqCst) {
        return;
    }
    let result = (|| -> anyhow::Result<(String, String, Vec<serde_json::Value>)> {
        let chat = if u.chat.is_empty() {
            s.store
                .create_chat(&u.input.trim().chars().take(48).collect::<String>())?
        } else {
            u.chat.clone()
        };
        let mut history = store::context(&s.store.messages(&chat)?);
        history.push(serde_json::json!({"role":"user","content":u.input.trim()}));
        s.store.add(&chat, "user", u.input.trim(), "complete", "")?;
        let id = s.store.add(
            &chat,
            "assistant",
            "",
            "streaming",
            &format!("{} {}", model.name, model.version),
        )?;
        Ok((chat, id, history))
    })();
    match result {
        Ok((chat, id, history)) => {
            s.cancel.store(false, Ordering::SeqCst);
            {
                let mut v = ui.write();
                v.chat = chat.clone();
                v.stream_chat = chat;
                v.stream_id = id;
                v.stream_text.clear();
                v.input.clear();
                v.busy = true;
                v.error.clear();
                v.status = "Modell wird vorbereitet …".into();
                if let Err(e) = v.refresh(&s) {
                    v.error = e.to_string();
                }
            }
            let tx = s.tx.clone();
            engine::run(
                s.root.clone(),
                kd_desktop::runtime_root(),
                model,
                history,
                u.backend,
                s.cancel.clone(),
                move |e| {
                    let _ = tx.send(Update::Engine(e));
                },
            );
        }
        Err(e) => {
            s.working.store(false, Ordering::SeqCst);
            failure(ui, e);
        }
    }
}

pub fn app() -> Element {
    #[cfg(feature = "ui-check")]
    use_future(|| async {
        let mut eval = document::eval(include_str!("../tests/navigation.js"));
        let mut result = eval
            .recv::<serde_json::Value>()
            .await
            .unwrap_or_else(|e| serde_json::json!({"passed":false,"error":e.to_string()}));
        result["build_profile"] = serde_json::json!(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        });
        if result["passed"] == true {
            let store = Store::open(&kd_desktop::data_root()).unwrap();
            result["saved_theme"] = serde_json::json!(store.setting("theme"));
            result["rename_persisted"] =
                serde_json::json!(!store.chats("Umbenannt").unwrap().is_empty());
            if result["saved_theme"] != "dark" || result["rename_persisted"] != true {
                result["passed"] = serde_json::json!(false);
            }
        }
        std::fs::write(
            kd_desktop::data_root().join("ui-result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        std::process::exit(if result["passed"] == true { 0 } else { 1 });
    });
    let init = use_hook(|| {
        let root = kd_desktop::data_root();
        let (tx, rx) = mpsc::channel();
        let store = Store::open(&root).expect("Lokale Chatdatenbank konnte nicht geöffnet werden");
        (
            Services {
                store,
                root,
                tx,
                cancel: Arc::new(AtomicBool::new(false)),
                working: Arc::new(AtomicBool::new(false)),
            },
            Rc::new(RefCell::new(rx)),
        )
    });
    let s = init.0;
    let receiver = init.1;
    let mut ui = use_signal(|| Ui::load(&s));
    use_context_provider(|| (s.clone(), ui));
    let polling = s.clone();
    use_future(move || {
        let receiver = receiver.clone();
        let s = polling.clone();
        async move {
            loop {
                let updates: Vec<_> = receiver.borrow().try_iter().collect();
                if !updates.is_empty() {
                    let mut u = ui.write();
                    for event in updates {
                        match event {
                            Update::ImportStatus(text) => u.status = text,
                            Update::Imported(result) => {
                                u.importing = false;
                                u.status.clear();
                                match result {
                                    Ok(model) => {
                                        u.active = model.id.clone();
                                        if let Err(e) =
                                            s.store.set_setting("active_model", &model.id)
                                        {
                                            u.error = e.to_string();
                                        }
                                        match models::list(&s.root) {
                                            Ok(m) => u.models = m,
                                            Err(e) => u.error = e.to_string(),
                                        }
                                    }
                                    Err(e) => u.error = e,
                                }
                            }
                            Update::Engine(event) => match event {
                                Event::Status(text) => u.status = text,
                                Event::Piece(piece) => u.stream_text.push_str(&piece),
                                Event::Finished(reason) => {
                                    u.busy = false;
                                    u.status = if reason == "length" {
                                        "Tokenlimit erreicht".into()
                                    } else {
                                        String::new()
                                    };
                                    if let Err(e) =
                                        s.store.update(&u.stream_id, &u.stream_text, "complete")
                                    {
                                        u.error = e.to_string();
                                    }
                                    s.working.store(false, Ordering::SeqCst);
                                }
                                Event::Error(error) => {
                                    u.busy = false;
                                    let status = if s.cancel.load(Ordering::SeqCst) {
                                        "stopped"
                                    } else {
                                        "error"
                                    };
                                    if status == "error" {
                                        u.error = error;
                                    }
                                    u.status.clear();
                                    if let Err(e) =
                                        s.store.update(&u.stream_id, &u.stream_text, status)
                                    {
                                        u.error = e.to_string();
                                    }
                                    s.working.store(false, Ordering::SeqCst);
                                }
                            },
                        }
                    }
                    if u.busy
                        && !u.stream_id.is_empty()
                        && let Err(e) = s.store.update(&u.stream_id, &u.stream_text, "streaming")
                    {
                        u.error = e.to_string();
                    }
                    if let Err(e) = u.refresh(&s) {
                        u.error = e.to_string();
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            }
        }
    });
    use_effect(move || {
        let theme = ui.read().theme.clone();
        if theme == "system" {
            spawn(async move {
                let mut eval = document::eval(
                    "dioxus.send(matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')",
                );
                if let Ok(effective) = eval.recv::<String>().await
                    && ui.read().theme == "system"
                {
                    ui.write().theme = effective;
                }
            });
            return;
        }
        let _ = document::eval(&format!(
            "document.documentElement.dataset.theme={};",
            serde_json::to_string(&theme).unwrap()
        ));
    });
    use_hook(|| {
        document::eval(include_str!("../assets/behavior.js"));
    });
    let u = ui.read().clone();
    rsx! {
        style { {include_str!("../assets/app.css")} }
        div {class:if u.sidebar{"app sidebar-open"}else{"app sidebar-closed"},
            // Dioxus 0.7 does not treat inert as a boolean attribute. In HTML,
            // inert="false" still disables the entire subtree: omit it when open.
            aside {class:"sidebar", "aria-label":"Chatnavigation",inert:if u.sidebar{None}else{Some("")},"aria-hidden":if u.sidebar{"false"}else{"true"},
                div {class:"sidebar-top",
                    button {class:"brand-mark",title:"Neuer Chat",onclick:move|_|fresh(ui),"kd"}
                    button {class:"icon-button",title:"Seitenleiste schliessen",onclick:move|_|ui.write().sidebar=false,Icon{name:"sidebar"}}
                }
                div {class:"nav-actions",
                    button {class:"nav-button",onclick:move|_|fresh(ui),Icon{name:"new"} "Neuer Chat" span{class:"shortcut","Ctrl N"}}
                    div {class:"search-wrap",Icon{name:"search"} input {placeholder:"Chats durchsuchen",value:"{u.search}","aria-label":"Chats durchsuchen",oninput:{let s=s.clone();move|e|{ui.write().search=e.value();let result=ui.write().refresh(&s);if let Err(e)=result{failure(ui,e)}}}}}
                    button {class:"nav-button",onclick:move|_|ui.write().panel="models".into(),Icon{name:"box"} "Modelle"}
                }
                div {class:"history", h2{"Deine Chats"}
                    if u.chats.is_empty(){p{class:"history-empty",if u.search.is_empty(){"Deine Gespräche erscheinen hier."}else{"Keine passenden Chats."}}}
                    for chat in &u.chats{
                        div {class:if chat.id==u.chat{"chat-row selected"}else{"chat-row"},key:"{chat.id}",
                            button {class:"chat-title",title:"{chat.title}",onclick:{let s=s.clone();let id=chat.id.clone();move|_|{ui.write().chat=id.clone();ui.write().input.clear();let result=ui.write().refresh(&s);if let Err(e)=result{failure(ui,e)}}},"{chat.title}"}
                            button {class:"chat-more",title:"Chat umbenennen oder löschen",onclick:{let id=chat.id.clone();let title=chat.title.clone();move|_|{let mut v=ui.write();v.rename_id=id.clone();v.rename_title=title.clone();v.panel="chat".into();}},Icon{name:"more"}}
                        }
                    }
                }
                div {class:"sidebar-bottom",
                    button {class:"profile-button",onclick:move|_|ui.write().panel="settings".into(),span{class:"profile-avatar","kd"} span{strong{"KD Desktop"} small{"Lokal auf deinem Computer"}} Icon{name:"settings"}}
                }
            }
            if u.sidebar{button{class:"sidebar-scrim","aria-label":"Seitenleiste schliessen",onclick:move|_|ui.write().sidebar=false}}
            main {
                header {class:"topbar",
                    div{class:"header-left",
                        if !u.sidebar{button{class:"icon-button",title:"Seitenleiste öffnen",onclick:move|_|ui.write().sidebar=true,Icon{name:"sidebar"}}}
                        span{class:"app-title","KD Desktop"}
                        div{class:"model-switcher",
                            button{class:"model-trigger",title:"Aktives Modell auswählen",disabled:u.busy||u.importing,onclick:move|_|{let open=ui.read().model_menu;ui.write().model_menu= !open;},span{class:"active-model-name","{u.model_label()}"} Icon{name:"chevron"}}
                            if u.model_menu{div{class:"model-dropdown",role:"menu",
                                p{class:"menu-label","Installierte Modelle"}
                                for model in &u.models{button{role:"menuitemradio","aria-checked":if model.id==u.active{"true"}else{"false"},onclick:{let s=s.clone();let id=model.id.clone();move|_|select_model(&s,ui,id.clone())},span{strong{"{model.name}"} small{"{model.version}"}} if model.id==u.active{Icon{name:"check"}}}}
                                button{class:"manage-link",onclick:move|_|{ui.write().model_menu=false;ui.write().panel="models".into();},Icon{name:"box"}"Modelle verwalten"}
                            }}
                        }
                    }
                    div{class:"header-right",span{class:"local-indicator",span{class:"status-dot"}"Lokal"}ThemeSwitch{settings:false}button{class:"icon-button",title:"Einstellungen",onclick:move|_|ui.write().panel="settings".into(),Icon{name:"settings"}}}
                }
                if u.models.is_empty(){
                    section{class:"first-run",
                        div{class:"welcome-logo","kd"}
                        h1{"Ein bisschen weniger wissen."}
                        p{class:"lead","Willkommen bei Künstliche Dummheit."}
                        p{class:"onboard-copy","Deine Chats, dein Modell, dein Computer. Importiere ein KD-Modellpaket, um dein erstes Gespräch zu beginnen."}
                        button{class:"primary-button",disabled:u.importing,onclick:{let s=s.clone();move|_|import(s.clone(),ui)},Icon{name:"upload"}if u.importing{"Modell wird importiert …"}else{"Modell-ZIP importieren"}}
                        p{class:"muted small",{if u.importing{u.status.clone()}else{"Die App enthält keine Modellgewichte. Alles bleibt lokal.".to_string()}}}
                    }
                }else{
                    div{class:if u.messages.is_empty(){"conversation empty"}else{"conversation"},id:"conversation",
                        if u.messages.is_empty(){div{class:"empty-state",h1{"Was möchtest du falsch verstehen?"}p{"Künstliche Dummheit. Ganz bei dir."}}}
                        div{class:"message-list",
                            for message in &u.messages{MessageView{key:"{message.id}",message:message.clone(),busy:u.busy&&message.id==u.stream_id}}
                            if u.busy && u.chat!=u.stream_chat{div{class:"background-note","Eine Antwort wird in einem anderen Chat geschrieben."}}
                        }
                    }
                    div{class:if u.messages.is_empty(){"composer-dock centered"}else{"composer-dock"},
                        div{class:"composer",
                            textarea{id:"prompt",placeholder:"Frag die KD irgendetwas",value:"{u.input}",rows:1,"aria-label":"Nachricht an KD",disabled:u.busy||u.importing,oninput:move|e|ui.write().input=e.value(),onkeydown:{let s=s.clone();move|e|{if e.key()==Key::Enter && !e.modifiers().shift(){e.prevent_default();submit(s.clone(),ui);}}}}
                            div{class:"composer-tools",
                                button{class:"composer-model",title:"Aktives Modell ändern",disabled:u.busy,onclick:move|_|ui.write().panel="models".into(),Icon{name:"box"}span{ "{u.model_label()}" }}
                                if u.busy{button{class:"send-button stop",title:"Antwort abbrechen",onclick:{let c=s.cancel.clone();move|_|{c.store(true,Ordering::SeqCst);ui.write().status="Antwort wird gestoppt …".into();}},Icon{name:"stop"}}}else{button{class:"send-button",title:"Nachricht senden",disabled:u.input.trim().is_empty()||u.importing,onclick:{let s=s.clone();move|_|submit(s.clone(),ui)},Icon{name:"arrow"}}}
                            }
                        }
                        if u.busy || !u.status.is_empty(){p{class:"generation-status",role:"status","{u.status}"}}
                        p{class:"disclaimer","KD erfindet Dinge. Absichtlich. Und manchmal auch unabsichtlich."}
                    }
                }
                if !u.error.is_empty(){div{class:"error-toast",role:"alert",span{"{u.error}"}button{class:"icon-button",title:"Meldung schliessen",onclick:move|_|ui.write().error.clear(),Icon{name:"close"}}}}
            }
            if !u.panel.is_empty(){Panel{}}
        }
    }
}

#[component]
fn MessageView(message: Message, busy: bool) -> Element {
    let content = message.content.clone();
    rsx! {article{class:if message.role=="user"{"message user-message"}else{"message assistant-message"},
        if message.role=="user"{div{class:"user-bubble","{message.content}"}}else{
            div{class:"assistant-content",
                if message.content.is_empty() && busy{div{class:"thinking-dot","aria-label":"Modell lädt"}}else{div{class:if busy{"markdown streaming"}else{"markdown"},dangerous_inner_html:kd_desktop::markdown::render(&message.content)}}
                if message.status=="stopped"{p{class:"message-state","Antwort abgebrochen"}}
                if message.status=="error"{p{class:"message-state","Antwort unvollständig – bitte erneut versuchen"}}
                if !busy{div{class:"message-actions",button{class:"icon-button",title:"Antwort kopieren",onclick:move|_|{let text=serde_json::to_string(&content).unwrap();document::eval(&format!("window.kdCopy({text})"));},Icon{name:"copy"}}span{class:"message-model","{message.model}"}}}
            }
        }
    }}
}

#[component]
fn Panel() -> Element {
    let (s, mut ui) = use_context::<(Services, Signal<Ui>)>();
    let u = ui.read().clone();
    rsx! {div{class:"modal-backdrop",onclick:move|_|ui.write().panel.clear(),onkeydown:move|e|{if e.key()==Key::Escape{ui.write().panel.clear();}},
        section{class:"modal",role:"dialog","aria-modal":"true","aria-labelledby":"panel-title",onclick:move|e|e.stop_propagation(),
            div{class:"modal-header",h2{id:"panel-title",match u.panel.as_str(){"models"=>"Modelle","settings"=>"Einstellungen","chat"=>"Chat bearbeiten",_=>"Löschen bestätigen"}}button{class:"icon-button",title:"Dialog schliessen",onclick:move|_|ui.write().panel.clear(),Icon{name:"close"}}}
            div{class:"modal-body",
                if u.panel=="models"{
                    p{class:"muted","Installiere verschiedene KD-Versionen und wähle das Modell für deine nächste Antwort."}
                    div{class:"model-list",
                        for model in &u.models{
                            div{class:if model.id==u.active{"model-item active"}else{"model-item"},
                                div{class:"model-item-icon",Icon{name:"box"}}
                                div{class:"model-details",strong{"{model.name}"}p{"{model.version} · " {format!("{:.2} GB",model.bytes as f64/1e9)}}small{title:"{model.sha256}","SHA-256 geprüft · GGUF"}}
                                button{class:"secondary-button",disabled:u.busy||u.importing||model.id==u.active,onclick:{let s=s.clone();let id=model.id.clone();move|_|select_model(&s,ui,id.clone())},if model.id==u.active{Icon{name:"check"}"Aktiv"}else{"Auswählen"}}
                                button{class:"icon-button danger",title:"Modell entfernen",disabled:u.busy||u.importing,onclick:{let id=model.id.clone();move|_|{let mut v=ui.write();v.delete_kind="model".into();v.delete_id=id.clone();v.panel="delete".into();}},Icon{name:"trash"}}
                            }
                        }
                    }
                    button{class:"primary-button",disabled:u.busy||u.importing,onclick:{let s=s.clone();move|_|import(s.clone(),ui)},Icon{name:"upload"}if u.importing{"Wird importiert …"}else{"Modell-ZIP importieren"}}
                    if u.importing{p{class:"muted",role:"status","{u.status}"}}
                    p{class:"muted small","Unterstützt KD-Modellpakete und bisherige KD-ZIPs mit einer GGUF-Datei. Importierte Modelle bleiben getrennt von der Anwendung."}
                }else if u.panel=="settings"{
                    div{class:"setting-row",label{r#for:"theme-switch","Dunkler Modus"}ThemeSwitch{settings:true}}
                    div{class:"setting-row",label{r#for:"backend","Berechnung"}select{id:"backend",value:"{u.backend}",disabled:u.busy,onchange:{let s=s.clone();move|e|{let value=e.value();if let Err(e)=s.store.set_setting("backend",&value){failure(ui,e)}else{ui.write().backend=value;}}},option{value:"cpu","CPU"}option{value:"cuda",disabled:!kd_desktop::runtime_root().join("cuda/llama-server.exe").exists(),"NVIDIA CUDA"}}}
                    p{class:"muted small","CUDA benötigt eine NVIDIA-Grafikkarte und die optionale CUDA-Laufzeit. CPU funktioniert ohne NVIDIA."}
                    div{class:"storage-info",strong{"Deine Daten bleiben hier"}code{"{s.root.display()}"}p{class:"muted small","Chats werden in SQLite gespeichert. Modelle liegen im Unterordner models. Bei der Deinstallation bleiben diese Daten standardmässig erhalten."}}
                    div{class:"about",strong{"KD Desktop"}span{"Version " {env!("CARGO_PKG_VERSION")} " · Rust & Dioxus"}p{class:"muted small","Die App-Version ist unabhängig vom gewählten Modell. Experimentelle Modellantworten können sprachliche Fehler und Wiederholungen enthalten."}UpdateStatus{}}
                }else if u.panel=="chat"{
                    label{r#for:"rename","Chatname"}input{id:"rename",class:"text-input",value:"{u.rename_title}",maxlength:160,oninput:move|e|ui.write().rename_title=e.value()}
                    div{class:"dialog-actions",button{class:"danger-button",disabled:u.busy&&u.rename_id==u.stream_chat,onclick:move|_|{let mut v=ui.write();v.delete_kind="chat".into();v.delete_id=v.rename_id.clone();v.panel="delete".into();},Icon{name:"trash"}"Chat löschen"}
                    button{class:"primary-button",onclick:{let s=s.clone();move|_|{let v=ui.read().clone();match s.store.rename(&v.rename_id,&v.rename_title){Ok(_)=>{ui.write().panel.clear();let result=ui.write().refresh(&s);if let Err(e)=result{failure(ui,e)}},Err(e)=>failure(ui,e)}}},"Speichern"}}
                }else if u.panel=="delete"{
                    p{if u.delete_kind=="model"{"Dieses installierte Modell wird vom Computer entfernt. Das ursprüngliche ZIP und deine Chats bleiben erhalten."}else{"Dieser Chat und alle zugehörigen Nachrichten werden dauerhaft gelöscht."}}
                    div{class:"dialog-actions",button{class:"secondary-button",onclick:move|_|ui.write().panel.clear(),"Abbrechen"}button{class:"danger-button",disabled:u.busy||u.importing,onclick:{let s=s.clone();move|_|{
                        let v=ui.read().clone();let result=if v.delete_kind=="model"{models::delete(&s.root,&v.delete_id)}else{s.store.delete_chat(&v.delete_id)};
                        match result{Ok(_)=>{let mut u=ui.write();u.panel.clear();if v.delete_kind=="model"{match models::list(&s.root){Ok(m)=>u.models=m,Err(e)=>u.error=e.to_string()};if u.active==v.delete_id{u.active=u.models.first().map(|m|m.id.clone()).unwrap_or_default();if let Err(e)=s.store.set_setting("active_model",&u.active){u.error=e.to_string();}}}else if u.chat==v.delete_id{u.chat.clear();u.messages.clear();}
                            if let Err(e)=u.refresh(&s){u.error=e.to_string();}},Err(e)=>failure(ui,e)}
                    }},"Dauerhaft löschen"}}
                }
            }
        }
    }}
}

#[component]
fn UpdateStatus() -> Element {
    #[cfg(windows)]
    {
        use kd_desktop::updater::Updater;
        let updater = try_consume_context::<Option<Updater>>().flatten();
        let mut status = use_signal(|| updater.as_ref().map(Updater::status));
        use_future(move || {
            let updater = updater.clone();
            async move {
                if let Some(updater) = updater {
                    loop {
                        let next = Some(updater.status());
                        if *status.read() != next {
                            status.set(next);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
        return rsx! {
            if let Some(message) = status.read().as_ref() {
                p { class: "muted small", role: "status", "{message}" }
            }
        };
    }
    #[cfg(not(windows))]
    rsx! {}
}

#[component]
fn ThemeSwitch(settings: bool) -> Element {
    let (s, ui) = use_context::<(Services, Signal<Ui>)>();
    let dark = ui.read().theme == "dark";
    rsx! {button{
        id:if settings{Some("theme-switch")}else{None},
        class:if settings{"theme-switch"}else{"theme-switch header-theme-switch"},
        role:"switch","aria-label":"Dunkler Modus","aria-checked":"{dark}",
        title:if dark{"Zum hellen Modus wechseln"}else{"Zum dunklen Modus wechseln"},
        onclick:move|_|set_theme(&s,ui,if dark{"light"}else{"dark"}),
        span{class:"switch-thumb",Icon{name:if dark{"moon"}else{"sun"}}}
    }}
}

#[component]
fn Icon(#[props(into)] name: String) -> Element {
    let path = match name.as_str() {
        "sidebar" => "M3 4h18v16H3z M9 4v16 M5.5 8h1 M5.5 12h1",
        "new" => {
            "M14 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-9 M10 14l1-4L19 2l3 3-8 8z"
        }
        "search" => "M10.5 3a7.5 7.5 0 1 0 0 15 7.5 7.5 0 0 0 0-15 M16 16l5 5",
        "box" => "M12 3l9 5v9l-9 5-9-5V8z M3 8l9 5 9-5 M12 13v9 M7.5 5.5l9 5",
        "chevron" => "M7 10l5 5 5-5",
        "arrow" => "M12 19V5 M5 12l7-7 7 7",
        "stop" => "M7 7h10v10H7z",
        "close" => "M6 6l12 12 M18 6 6 18",
        "check" => "M5 12l4 4L19 6",
        "more" => "M4 12h.1 M12 12h.1 M20 12h.1",
        "copy" => "M9 9h11v12H9z M15 5V2H3v13h3",
        "trash" => "M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7",
        "upload" => "M12 16V3 M7 8l5-5 5 5 M4 15v6h16v-6",
        "sun" => {
            "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l1.5 1.5 M17.5 17.5 19 19 M5 19l1.5-1.5 M17.5 6.5 19 5"
        }
        "moon" => "M20 15.5A9 9 0 0 1 8.5 4 9 9 0 1 0 20 15.5z",
        _ => {
            "M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8"
        }
    };
    rsx! {svg{width:"20",height:"20",view_box:"0 0 24 24",fill:"none",stroke:"currentColor",stroke_width:"1.65",stroke_linecap:"round",stroke_linejoin:"round","aria-hidden":"true",path{d:path}}}
}
