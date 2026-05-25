use std::collections::{HashMap, HashSet};

use macroquad::miniquad::conf::{Platform, WebGLVersion};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

const SAVE_KEY: &str = "klixx_rust_save_v11_archive_case";
const VW: f32 = 1280.0;
const VH: f32 = 720.0;
const INVENTORY_X: f32 = 540.0;
const INVENTORY_Y: f32 = 620.0;
const INVENTORY_SLOT: f32 = 34.0;
const INVENTORY_GAP: f32 = 5.0;
const INVENTORY_COLUMNS: usize = 9;
const PLAYER_DRAW_W: f32 = 180.0;
const PLAYER_DRAW_H: f32 = 240.0;
const PLAYER_FOOT_ANCHOR_Y: f32 = 0.97;
const PLAYER_DEPTH_MIN_SCALE: f32 = 0.68;
const PLAYER_DEPTH_MAX_SCALE: f32 = 1.0;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Verb {
    Look,
    Poke,
    Use,
    PickUp,
    Tongue,
    Smell,
    Talk,
}

impl Verb {
    const ALL: [Verb; 7] = [
        Verb::Look,
        Verb::Poke,
        Verb::Use,
        Verb::PickUp,
        Verb::Tongue,
        Verb::Smell,
        Verb::Talk,
    ];

    fn label(self) -> &'static str {
        match self {
            Verb::Look => "Ansehen",
            Verb::Poke => "Anstubsen",
            Verb::Use => "Benutzen",
            Verb::PickUp => "Aufheben",
            Verb::Tongue => "Anzüngeln",
            Verb::Smell => "Riechen",
            Verb::Talk => "Reden",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Verb::Look => "Ansehen: prüft Details und dokumentiert Hinweise.",
            Verb::Poke => "Anstubsen: bewegt oder testet lose Teile.",
            Verb::Use => "Benutzen: bedient Schalter, Türen, Terminals und vorbereitete Kombinationen.",
            Verb::PickUp => "Aufheben: nimmt lose Gegenstände ins Inventar.",
            Verb::Tongue => "Anzüngeln: probiert Oberflächen mit der Zunge. Meistens ist das eine schlechte Idee.",
            Verb::Smell => "Riechen: prüft Material, Feuchtigkeit oder Luft.",
            Verb::Talk => "Reden: spricht mit erreichbaren Personen.",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct GameState {
    scene: String,
    verb: Verb,
    selected_item: Option<String>,
    inventory: Vec<String>,
    flags: HashSet<String>,
    log: Vec<String>,
    complete: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            scene: "greenscreen_studio".to_string(),
            verb: Verb::Look,
            selected_item: None,
            inventory: Vec::new(),
            flags: HashSet::new(),
            log: vec![
                "Fall 000 ist offen: Ein Host ist in einem geöffneten Klixx-Frame verschwunden. Die Regie braucht stabile Gegenwart, Stadtzeit und Clip-Anker, bevor der Rettungslauf starten kann."
                    .to_string(),
            ],
            complete: false,
        }
    }
}

struct Game {
    state: GameState,
    scene_textures: HashMap<&'static str, Texture2D>,
    player_texture: Option<Texture2D>,
    inventory_icons: Option<Texture2D>,
    verb_icons: Option<Texture2D>,
    status: String,
    hover: Option<String>,
    modal: Modal,
    dialogue_response: Option<DialogueResponse>,
    player_pos: Vec2,
    walk_target: Option<Vec2>,
    player_facing: PlayerFacing,
    death: Option<DeathState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Modal {
    None,
    Dialogue(&'static str),
    Milestone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayerFacing {
    Down,
    Left,
    Right,
    Up,
}

impl PlayerFacing {
    fn row(self) -> f32 {
        match self {
            PlayerFacing::Down => 0.0,
            PlayerFacing::Left => 1.0,
            PlayerFacing::Right => 2.0,
            PlayerFacing::Up => 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeathKind {
    Shock,
    Fire,
    Fall,
    Signal,
}

impl DeathKind {
    fn destination(self) -> &'static str {
        match self {
            DeathKind::Shock => "hospital_room",
            DeathKind::Fire => "sprinkler_courtyard",
            DeathKind::Fall => "prop_storage_collapse",
            DeathKind::Signal => "archive_recovery",
        }
    }

    fn arrival_status(self) -> &'static str {
        match self {
            DeathKind::Shock => {
                "Du wachst im Krankenhaus auf. Der Kontakt mit dem Rack war echt, nicht symbolisch."
            }
            DeathKind::Fire => {
                "Der Alarm ist real. Die Produktion steht nass im Evakuierungsbereich."
            }
            DeathKind::Fall => {
                "Du kommst zwischen Requisitenteilen zu dir. Der Rückweg muss gesichert werden."
            }
            DeathKind::Signal => {
                "Du kommst im Archiv-Rettungsplatz wieder zu dir. Der beschädigte Videoeintrag liegt dort auf der Diagnose-Station."
            }
        }
    }
}

#[derive(Clone, Debug)]
struct DeathState {
    kind: DeathKind,
    started_at: f64,
}

struct DialogueResponse {
    dialogue_id: &'static str,
    text: &'static str,
}

#[derive(Clone, Copy)]
struct SceneMeta {
    id: &'static str,
    name: &'static str,
    zone: &'static str,
    token: (f32, f32),
    walkable: &'static [(f32, f32)],
    hotspots: &'static [HotspotSpec],
}

#[derive(Clone, Copy)]
struct HotspotSpec {
    id: &'static str,
    name: &'static str,
    pct: Rect,
    kind: HotspotKind,
    #[allow(dead_code)]
    look: &'static str,
    inspect: &'static str,
    talk_id: Option<&'static str>,
}

struct HotspotPolygonSpec {
    scene_id: &'static str,
    hotspot_id: &'static str,
    points: &'static [(f32, f32)],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HotspotKind {
    Character,
    Pickup,
    Prop,
    Exit,
}

struct ItemMeta {
    id: &'static str,
    name: &'static str,
    short: &'static str,
    description: &'static str,
}

struct Dialogue {
    id: &'static str,
    speaker: &'static str,
    opening: &'static str,
    choices: &'static [DialogueChoice],
}

struct DialogueChoice {
    label: &'static str,
    response: &'static str,
    flag: &'static str,
    log: &'static str,
}

const fn pct(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

const STUDIO_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "office_hall",
        name: "Büroflur",
        pct: pct(5.43, 25.47, 6.09, 42.92),
        kind: HotspotKind::Exit,
        look: "Die Tür zum Büroflur steht offen. Dahinter hängt die Falltafel, viel zu ordentlich für einen verschwundenen Host.",
        inspect: "Im Flur hängen Tagesplan, Raumbelegung und die erste Chronologie von Folge 000.",
        talk_id: None,
    },
    HotspotSpec {
        id: "control_room",
        name: "Regie",
        pct: pct(94.43, 26.61, 4.85, 51.83),
        kind: HotspotKind::Exit,
        look: "Hinter der Scheibe liegt die Regie. Auf der Monitorwand wartet ein Standbild, das zu tief wirkt.",
        inspect: "Dort laufen Bild, Ton, Grafik und der geöffnete Archivframe zusammen. Die Signalroute muss eindeutig beschriftet sein.",
        talk_id: None,
    },
    HotspotSpec {
        id: "greenscreen_wall",
        name: "Greenscreen-Fläche",
        pct: pct(38.65, 16.75, 32.76, 47.22),
        kind: HotspotKind::Prop,
        look: "Die grüne Fläche füllt den hinteren Teil des Studios. Sie ist weniger Hintergrund als leere Zeit.",
        inspect: "Der Stoff ist sauber gespannt. Damit der Host zurück in die Gegenwart fallen kann, fehlen neue Bodenmarken.",
        talk_id: None,
    },
    HotspotSpec {
        id: "floor_marks",
        name: "Bodenmarken",
        pct: pct(38.87, 63.25, 22.0, 9.0),
        kind: HotspotKind::Prop,
        look: "Alte Tape-Reste markieren Positionen aus Sendungen, die schon vorbei sein sollten.",
        inspect: "Für den Rettungslauf braucht das Studio klare Standpunkte. Sonst landet jemand im falschen Take.",
        talk_id: None,
    },
    HotspotSpec {
        id: "klixx_table",
        name: "Klixx-Tisch",
        pct: pct(23.54, 51.11, 16.68, 24.39),
        kind: HotspotKind::Prop,
        look: "Ein schmaler Tisch steht vor der Greenscreen-Fläche. Darauf klebt noch ein Zettel: keine Tipps nach der Auflösung.",
        inspect: "Die Oberfläche ist frei. Die spätere Kameraposition hängt von den Bodenmarken und vom stabilen Frame-Anker ab.",
        talk_id: None,
    },
    HotspotSpec {
        id: "chat_preview",
        name: "Chat-Vorschau",
        pct: pct(19.94, 29.56, 4.57, 18.36),
        kind: HotspotKind::Prop,
        look: "Ein Testmonitor zeigt Platzhalter für Chat, Host-Karte und eine merkwürdig selbstsichere Klickzahl.",
        inspect: "Die Vorschau enthält noch keine Host-Karte. Am Grafikplatz fehlt die freigegebene Karte mit dem richtigen Host-Namen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "camera_one",
        name: "Studiokamera",
        pct: pct(62.34, 41.44, 10.75, 34.69),
        kind: HotspotKind::Prop,
        look: "Die Kamera ist auf Tisch und Greenscreen eingerichtet. Ihr Sucher zeigt eine halbe Sekunde Zukunft.",
        inspect: "Der Bildausschnitt ist eng. Licht, Standpositionen und der Clip-Anker müssen vor dem Rettungslauf geprüft werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mentor_shadow",
        name: "Aufnahmeleitung",
        pct: pct(75.74, 37.92, 4.67, 34.25),
        kind: HotspotKind::Character,
        look: "Die Aufnahmeleitung steht mit Headset, Klemmbrett und dem Gesichtsausdruck einer Person, die gerade Zeitreisen in den Drehplan schreiben musste.",
        inspect: "Sie koordiniert Raumbelegung, Zeitplan, Rückmeldungen aus der Regie und alles, was nicht offiziell Paradox heißt.",
        talk_id: Some("mentor"),
    },
    HotspotSpec {
        id: "greenscreen_setup",
        name: "Rückhol-Setup",
        pct: pct(49.18, 35.89, 23.0, 37.0),
        kind: HotspotKind::Prop,
        look: "Hier laufen Kamera, Greenscreen und Regiesignal zusammen. Wenn der Host zurückkommt, dann genau hier.",
        inspect: "Der Rettungslauf braucht Fallakte, Bodenmarken, Signalweg, Grafik, stabilisierten Clip, Timing und eine Lichtreferenz.",
        talk_id: None,
    },
];

const OFFICE_HALL_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "building_courtyard",
        name: "Hof",
        pct: pct(2.49, 21.81, 6.58, 46.86),
        kind: HotspotKind::Exit,
        look: "Die Tür führt in den Hof zwischen den Häusern.",
        inspect: "Vom Hof aus sind Haus 15, die Ladezone und der Ausgang zur Straße erreichbar.",
        talk_id: None,
    },
    HotspotSpec {
        id: "greenscreen_studio",
        name: "Greenscreen-Studio",
        pct: pct(16.33, 25.78, 6.92, 41.67),
        kind: HotspotKind::Exit,
        look: "Die Studiotür führt zurück zum Greenscreen-Aufbau.",
        inspect: "Das On-Air-Licht ist aus. Der Raum wartet auf den Rettungslauf.",
        talk_id: None,
    },
    HotspotSpec {
        id: "control_room",
        name: "Regie",
        pct: pct(34.16, 26.25, 6.25, 40.89),
        kind: HotspotKind::Exit,
        look: "Die Regietür ist nur angelehnt.",
        inspect: "Durch die Scheibe sieht man den Greenscreen auf mehreren Monitoren.",
        talk_id: None,
    },
    HotspotSpec {
        id: "server_room",
        name: "Serverraum 11",
        pct: pct(84.65, 27.42, 5.05, 40.56),
        kind: HotspotKind::Exit,
        look: "Der Weg zum Serverraum in Nummer 11.",
        inspect: "Dort beginnt die lange SDI-Strecke zurück ins Studio.",
        talk_id: None,
    },
    HotspotSpec {
        id: "schanzenstrasse",
        name: "Raus zur Schanze",
        pct: pct(73.18, 27.39, 5.92, 37.31),
        kind: HotspotKind::Exit,
        look: "Ein Ausgang führt aus dem Gebäude Richtung Schanzenviertel.",
        inspect: "Die Fallakte nennt ein Außenmotiv im Schanzenviertel.",
        talk_id: None,
    },
    HotspotSpec {
        id: "dispo_board",
        name: "Falltafel 000",
        pct: pct(24.53, 31.94, 8.02, 22.67),
        kind: HotspotKind::Pickup,
        look: "Farbcodes, Zeiten und Raumnummern füllen die Tafel. Ein roter Faden verbindet Regie, Stadt und einen Schimmelbrüder-Frame.",
        inspect: "Dein Name steht neben Fall 000: Gegenwart markieren, Signal benennen, Stadtzeit sichern, Clip-Anker stabilisieren.",
        talk_id: None,
    },
    HotspotSpec {
        id: "equipment_storage",
        name: "Techniklager",
        pct: pct(43.54, 26.25, 13.0, 40.83),
        kind: HotspotKind::Prop,
        look: "Kamerataschen, Kabel, Mikrofone und Kleinteile stehen griffbereit. Die Schublade für Zeitreisen ist leer beschriftet.",
        inspect: "Die Fächer sind beschriftet. Für Fall 000 sind nur die vorhandenen Anker relevant, nicht noch mehr Technik.",
        talk_id: None,
    },
    HotspotSpec {
        id: "staircase",
        name: "Treppe zum Schlauchbüro",
        pct: pct(60.8, 13.11, 8.98, 50.06),
        kind: HotspotKind::Prop,
        look: "Eine Treppe hinauf Richtung Morning Call und langes Büro.",
        inspect: "Die Treppe führt in die Büros. Für Fall 000 ist der Weg nicht relevant.",
        talk_id: None,
    },
    HotspotSpec {
        id: "route_map",
        name: "Gebäudeplan",
        pct: pct(92.8, 31.69, 6.51, 21.64),
        kind: HotspotKind::Prop,
        look: "Ein Plan der Häuser 9, 11 und 15. Jemand hat eine rote Linie eingezeichnet, dann wieder ausradiert, dann wieder eingezeichnet.",
        inspect: "Studio, Serverraum und Szenenbau sind eingezeichnet. Der Außenstandort fehlt, als hätte die Stadt erst später in den Fall eingegriffen.",
        talk_id: None,
    },
];

const COURTYARD_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "office_hall",
        name: "Büroeingang",
        pct: pct(50.44, 31.58, 7.45, 31.78),
        kind: HotspotKind::Exit,
        look: "Der Eingang zurück ins Bürogebäude.",
        inspect: "Der Eingang führt zurück zum Flur mit Dispo-Board und Raumplan.",
        talk_id: None,
    },
    HotspotSpec {
        id: "set_workshop",
        name: "Haus-15-Szenenbau",
        pct: pct(80.72, 33.5, 5.17, 33.92),
        kind: HotspotKind::Exit,
        look: "Haus 15, ehemaliger Werkstattbereich, heute Kulissen und Setbau.",
        inspect: "Dort lagern Kulissenteile, Werkzeuge und Klebeband.",
        talk_id: None,
    },
    HotspotSpec {
        id: "schanzenstrasse",
        name: "Ausgang zur Straße",
        pct: pct(7.67, 25.97, 7.75, 47.33),
        kind: HotspotKind::Exit,
        look: "Ein Ausgang führt in Richtung Schanzenviertel.",
        inspect: "Der Ausgang führt vom Produktionsgelände auf die Straße.",
        talk_id: None,
    },
    HotspotSpec {
        id: "loading_zone",
        name: "Ladezone",
        pct: pct(19.56, 46.33, 18.0, 25.69),
        kind: HotspotKind::Prop,
        look: "Flightcases und Paletten stehen am Rand des Hofs.",
        inspect: "Die Beschriftungen gehören zu früheren Produktionen. Für die aktuelle Probe sind sie nicht eingeplant.",
        talk_id: None,
    },
    HotspotSpec {
        id: "address_plate",
        name: "Hausnummern",
        pct: pct(68.55, 30.78, 4.04, 12.81),
        kind: HotspotKind::Prop,
        look: "Ein Adressschild für die Heinrichstraße.",
        inspect: "Die Häuser 9, 11 und 15 liegen nah beieinander. Der technische Signalweg führt trotzdem über mehrere Räume.",
        talk_id: None,
    },
];

const SERVER_ROOM_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "office_hall",
        name: "Büroflur",
        pct: pct(4.35, 25.69, 5.51, 45.89),
        kind: HotspotKind::Exit,
        look: "Zurück in den Flur.",
        inspect: "Der Flur verbindet Serverraum, Regie und Studio.",
        talk_id: None,
    },
    HotspotSpec {
        id: "server_racks",
        name: "Serverracks",
        pct: pct(17.17, 19.3, 21.87, 52.86),
        kind: HotspotKind::Prop,
        look: "Mehrere Serverracks stehen entlang der Wand.",
        inspect: "Die Racks versorgen interne Tools, Schnittplätze und Signalverteilung.",
        talk_id: None,
    },
    HotspotSpec {
        id: "video_hub",
        name: "Video-Hub",
        pct: pct(41.82, 21.31, 19.34, 49.67),
        kind: HotspotKind::Prop,
        look: "Die Kreuzschiene verteilt Signale durch Räume, Studios und heute leider auch durch offene Frames.",
        inspect: "Die Route von Nummer 11 ins Studio ist aktiv. Sie braucht ein lesbares Label, damit der Rückweg nicht im Archiv rauscht.",
        talk_id: None,
    },
    HotspotSpec {
        id: "sdi_spool",
        name: "80-m-SDI-Rolle",
        pct: pct(70.59, 51.56, 13.15, 23.97),
        kind: HotspotKind::Prop,
        look: "Eine schwere Rolle SDI-Kabel liegt am Boden. Achtzig Meter Gegenwart auf Plastik gewickelt.",
        inspect:
            "Die Kabellänge reicht bis ins Studio. Die Strecke muss eindeutig beschriftet werden, bevor jemand ihr blind folgt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "sdi_label_printer",
        name: "Labeldrucker",
        pct: pct(62.8, 41.41, 5.2, 9.53),
        kind: HotspotKind::Pickup,
        look: "Ein kleiner Labeldrucker steht neben dem Patchfeld und spuckt Etiketten aus, als seien Namen Schutzzauber.",
        inspect: "Das vorbereitete Etikett benennt die Greenscreen-Route. Ohne Namen ist jedes Kabel nur eine Ausrede.",
        talk_id: None,
    },
];

const SET_WORKSHOP_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "building_courtyard",
        name: "Hof",
        pct: pct(4.19, 27.92, 5.02, 44.78),
        kind: HotspotKind::Exit,
        look: "Zurück in den Hof zwischen den Häusern.",
        inspect: "Von dort geht es zurück in den Gebäudekomplex.",
        talk_id: None,
    },
    HotspotSpec {
        id: "set_pieces",
        name: "Kulissenteile",
        pct: pct(14.65, 13.39, 24.0, 56.89),
        kind: HotspotKind::Prop,
        look: "Kulissenteile lehnen dicht an der Wand.",
        inspect: "Die Kulissenteile sind beschriftet und eingelagert. Für Fall 000 zählt hier nur das Tape.",
        talk_id: None,
    },
    HotspotSpec {
        id: "gaffer_roll",
        name: "Gaffer-Tape",
        pct: pct(57.0, 51.33, 3.27, 6.17),
        kind: HotspotKind::Pickup,
        look: "Eine Rolle Gaffer-Tape liegt auf dem Arbeitstisch.",
        inspect: "Das Tape ist breit genug für neue Bodenmarken im Studio.",
        talk_id: None,
    },
    HotspotSpec {
        id: "bike_workshop_trace",
        name: "Werkstattspur",
        pct: pct(67.49, 13.28, 16.0, 22.0),
        kind: HotspotKind::Prop,
        look: "Alte Spuren der Fahrradwerkstatt im Boden und an der Wand.",
        inspect: "Die Spuren stammen aus einer früheren Nutzung des Gebäudes.",
        talk_id: None,
    },
    HotspotSpec {
        id: "studio_door_15",
        name: "Zusatzstudio",
        pct: pct(85.53, 29.31, 7.48, 37.97),
        kind: HotspotKind::Prop,
        look: "Eine weitere Studiotür im Haus-15-Bereich.",
        inspect: "Der Raum ist belegt. Für die aktuelle Aufgabe ist kein Zugang vorgesehen.",
        talk_id: None,
    },
];

const CONTROL_ROOM_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "office_hall",
        name: "Büroflur",
        pct: pct(3.39, 23.92, 7.33, 44.64),
        kind: HotspotKind::Exit,
        look: "Zurück in den Flur.",
        inspect: "Der Flur führt zurück zu Dispo-Board, Serverraum und Hof.",
        talk_id: None,
    },
    HotspotSpec {
        id: "greenscreen_studio",
        name: "Studiofenster",
        pct: pct(22.16, 15.14, 24.75, 28.89),
        kind: HotspotKind::Exit,
        look: "Ein Rückweg ins Greenscreen-Studio.",
        inspect: "Im Vorschaubild fehlen Bodenmarken und Lichtreferenz.",
        talk_id: None,
    },
    HotspotSpec {
        id: "rehearsal_monitor",
        name: "Anker-Monitor",
        pct: pct(32.57, 46.47, 3.46, 7.31),
        kind: HotspotKind::Prop,
        look: "Der Monitor zeigt das Studio, den leeren Hintergrundkanal und einen Host-Schatten, der immer einen Frame zu spät ist.",
        inspect: "Bild und Ton liegen an. Für den Rückholpunkt fehlen Grafikdaten, Stadtbezug und der stabile Schimmelbrüder-Frame.",
        talk_id: None,
    },
    HotspotSpec {
        id: "graphics_terminal",
        name: "Grafikplatz",
        pct: pct(66.11, 42.47, 12.64, 26.81),
        kind: HotspotKind::Prop,
        look: "Der Grafikplatz ist für Host-Karten, Einblendungen und die Identität des vermissten Hosts vorbereitet.",
        inspect: "Im System ist ein leerer Platzhalter. Die freigegebene Karte fehlt, und ohne Namen findet der Frame niemanden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "intercom_voice",
        name: "Regiestimme",
        pct: pct(56.29, 43.81, 6.9, 16.53),
        kind: HotspotKind::Character,
        look: "Eine Stimme aus der Gegensprechanlage.",
        inspect: "Die Gegensprechanlage ist offen. Die Regie wartet auf den vollständigen Rettungslauf.",
        talk_id: Some("mentor"),
    },
    HotspotSpec {
        id: "on_air_lamp",
        name: "On-Air-Lampe",
        pct: pct(87.66, 10.25, 4.34, 6.0),
        kind: HotspotKind::Prop,
        look: "Die On-Air-Lampe ist noch dunkel.",
        inspect: "Wenn sie leuchtet, sollte niemand mehr im Bild arbeiten.",
        talk_id: None,
    },
    HotspotSpec {
        id: "video_schimmelbrueder",
        name: "Schimmelbrüder-Frame",
        pct: pct(63.58, 33.36, 3.9, 7.94),
        kind: HotspotKind::Exit,
        look: "Der Monitor zeigt eine Fertigungshalle aus dem Schimmelbrüder-Video. Das Standbild flackert nicht wie eine Aufnahme, sondern wie ein Datum.",
        inspect: "Die Regie kann diesen Frame öffnen. Die Figur muss hinein, weil der Host-Echo dort an Musterbahn und Frame-Probe hängt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "video_icemachine",
        name: "Eismaschinen-Frame",
        pct: pct(68.2, 33.36, 3.9, 7.94),
        kind: HotspotKind::Exit,
        look: "Ein Archivmonitor zeigt die Eismaschine aus dem Feedback-Video. Der Raum wirkt geputzt, aber noch nicht abgeschlossen.",
        inspect: "Der Frame ist als neuer Videoraum markiert. Die Auswahlnotiz sagt: nicht wirklich begehbar, eher ein sauberer Inspektionsraum.",
        talk_id: None,
    },
    HotspotSpec {
        id: "video_brassband",
        name: "Band-Frame",
        pct: pct(72.82, 33.36, 3.9, 7.94),
        kind: HotspotKind::Exit,
        look: "Ein weiterer Monitor hält eine Band mitten im Spielen fest. Das Standbild hat mehr Publikum im Ton als im Bild.",
        inspect: "Dieser Frame ist als begehbarer Videoraum freigegeben. Die Instrumente sollten genug Szene haben, um darin zu stöbern.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mixing_console",
        name: "Mischpult",
        pct: pct(82.71, 43.58, 17.11, 22.39),
        kind: HotspotKind::Prop,
        look: "Das Mischpult füllt den rechten Regieplatz. Die Fader stehen wie eine kleine Stadt aus Schiebereglern.",
        inspect: "Ton, Talkback und Archivkanal liegen hier an. Für Fall 000 ist wichtig: Der geöffnete Frame darf nicht auf den Studio-Rückweg gemischt werden.",
        talk_id: None,
    },
];

const VIDEO_SCHIMMEL_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "control_room",
        name: "Zurück zur Regie",
        pct: pct(1.32, 21.94, 6.62, 16.25),
        kind: HotspotKind::Exit,
        look: "Links bleibt die Rückverbindung zur Regie sichtbar.",
        inspect: "Dieser Bereich beendet den geöffneten Videoraum und führt zur Regie zurück.",
        talk_id: None,
    },
    HotspotSpec {
        id: "casting_table",
        name: "Gießtisch",
        pct: pct(6.18, 12.56, 12.9, 40.64),
        kind: HotspotKind::Prop,
        look: "Eine lange Arbeitsfläche zieht sich in den linken Bildrand. Jeder Fleck wirkt wie ein Standbild aus einem anderen Versuch.",
        inspect: "Die roten Markierungen kennzeichnen wiederkehrende Arbeitsschritte an der Gusslinie. Nichts daran ist freundlich zu Fingern.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mold_rack",
        name: "Formenreihe",
        pct: pct(19.61, 27.44, 21.06, 39.05),
        kind: HotspotKind::Prop,
        look: "Mehrere runde Formen stehen in zwei Reihen. In einer spiegelt sich kurz das Studio, obwohl es hier nicht sein kann.",
        inspect: "Die Formen unterscheiden sich in Füllstand und Helligkeit. Die Reihenfolge läuft von links nach rechts: leer, voll, leer, voll.",
        talk_id: None,
    },
    HotspotSpec {
        id: "patterned_belt",
        name: "Musterbahn",
        pct: pct(55.0, 17.0, 31.0, 42.0),
        kind: HotspotKind::Prop,
        look: "Auf der rechten Bahn liegt ein helles Zickzackmuster. Es sieht weniger gedruckt als gespeichert aus.",
        inspect: "Das Muster entspricht der Reihenfolge der Formen: leer, voll, leer, voll. Der Ausgang akzeptiert nur diese saubere Zuordnung.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mold_token",
        name: "Frame-Probe",
        pct: pct(41.3, 80.56, 8.97, 13.22),
        kind: HotspotKind::Pickup,
        look: "Ein einzelnes helles Stück liegt am Rand der Gusslinie. Es hält die Form, aber nicht ganz die Gegenwart.",
        inspect: "Die Probe ist ein bewegliches Teil aus der Formenreihe. Sie trägt genug Materialspur, um den Host-Echo am Ausgang zu kalibrieren.",
        talk_id: None,
    },
    HotspotSpec {
        id: "song_exit_gate",
        name: "Clip-Ausgang",
        pct: pct(86.48, 29.36, 13.35, 53.11),
        kind: HotspotKind::Prop,
        look: "Am rechten Rand liegt der Ausgang aus dem geöffneten Clip. Dahinter ist nicht schwarz, sondern Regielicht.",
        inspect: "Der Ausgang ist mit der Musterbahn und der Frame-Probe verknüpft. Ohne Zuordnung bleibt er nur ein Schnitt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "factory_floor",
        name: "Hallenboden",
        pct: pct(7.57, 66.0, 79.24, 34.0),
        kind: HotspotKind::Prop,
        look: "Der Hallenboden ist zwischen Formenreihe und Musterbahn begehbar. Jeder Schritt klingt leicht zeitversetzt.",
        inspect: "Der Frame enthält eine durchgehende Standfläche vor der Formenreihe. Gut: Man kann darin gehen. Schlecht: Man kann darin bleiben.",
        talk_id: None,
    },
];

const VIDEO_ICEMACHINE_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "control_room",
        name: "Zurück zur Regie",
        pct: pct(3.31, 17.11, 6.67, 46.22),
        kind: HotspotKind::Exit,
        look: "Links bleibt die Rückverbindung zur Regie offen.",
        inspect: "Der Videoraum ist nur als Standbild stabil. Ein Schritt zurück führt wieder auf die Monitore.",
        talk_id: None,
    },
    HotspotSpec {
        id: "ice_machine",
        name: "Eismaschine",
        pct: pct(51.02, 8.53, 19.4, 64.31),
        kind: HotspotKind::Prop,
        look: "Die Eismaschine steht im Zentrum des Frames, halb Gerät, halb Verdächtige.",
        inspect: "Jede gereinigte Kante glänzt minimal anders. Hier wurde nicht nur sauber gemacht, sondern ein Zustand eingefroren.",
        talk_id: None,
    },
    HotspotSpec {
        id: "cleaning_bucket",
        name: "Reinigungseimer",
        pct: pct(11.84, 49.22, 12.48, 28.06),
        kind: HotspotKind::Prop,
        look: "Ein Eimer mit Putzzeug steht im unteren Bildbereich.",
        inspect: "Die Mittel riechen scharf genug, um jeden Kommentar aus dem Clip zu lösen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "service_counter",
        name: "Arbeitsfläche",
        pct: pct(64.38, 39.83, 35.62, 17.89),
        kind: HotspotKind::Prop,
        look: "Die Arbeitsfläche sammelt Lappen, Schalen und ein paar sehr praktische Schatten.",
        inspect: "Auf der Fläche liegen keine Schlüssel, aber klare Hinweise auf eine wiederholte Reinigungsszene.",
        talk_id: None,
    },
];

const VIDEO_BRASSBAND_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "control_room",
        name: "Zurück zur Regie",
        pct: pct(4.17, 19.59, 7.85, 45.08),
        kind: HotspotKind::Exit,
        look: "Am linken Rand bleibt der Schnitt zurück in die Regie sichtbar.",
        inspect: "Der Ausgang hängt wie ein stiller Taktstrich am Bildrand.",
        talk_id: None,
    },
    HotspotSpec {
        id: "brass_players",
        name: "Bläsergruppe",
        pct: pct(39.74, 32.17, 41.55, 34.53),
        kind: HotspotKind::Character,
        look: "Die Band steht mitten im Einsatz. Niemand bewegt sich, aber die Haltung sieht nach dem nächsten Takt aus.",
        inspect: "Man kann fast erkennen, wer gerade Luft holt und wer nur so tut.",
        talk_id: None,
    },
    HotspotSpec {
        id: "tuba_bell",
        name: "Tuba",
        pct: pct(57.84, 25.61, 5.8, 21.53),
        kind: HotspotKind::Prop,
        look: "Der Trichter der Tuba fängt Licht wie ein zweiter Monitor.",
        inspect: "In der Spiegelung liegt ein Stück Regieblau. Der Frame ist also nicht ganz von hier.",
        talk_id: None,
    },
    HotspotSpec {
        id: "music_stand",
        name: "Notenständer",
        pct: pct(45.48, 46.92, 4.51, 20.33),
        kind: HotspotKind::Prop,
        look: "Ein Notenständer hält den Ablauf, auch wenn der Clip selbst pausiert.",
        inspect: "Die Noten sehen eher wie Schnittmarken aus: kurz, kurz, lang, Ausgang.",
        talk_id: None,
    },
    HotspotSpec {
        id: "stage_floor",
        name: "Bodenmarke",
        pct: pct(4.27, 79.03, 72.0, 18.0),
        kind: HotspotKind::Prop,
        look: "Der Boden vor der Band ist frei genug für ein paar vorsichtige Schritte.",
        inspect: "Die freie Fläche trägt keine Kabel, aber eine klare Standposition für die Figur.",
        talk_id: None,
    },
];

const SCHANZENSTRASSE_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "building_courtyard",
        name: "Zur Heinrichstraße",
        pct: pct(3.12, 38.39, 6.98, 32.5),
        kind: HotspotKind::Exit,
        look: "Der Weg zurück zum Produktionshof.",
        inspect: "Der Produktionshof ist über die Heinrichstraße erreichbar.",
        talk_id: None,
    },
    HotspotSpec {
        id: "sternschanze_station",
        name: "Sternschanze",
        pct: pct(73.14, 31.94, 26.86, 39.17),
        kind: HotspotKind::Exit,
        look: "Die Straße führt weiter zum Bahnhof Sternschanze.",
        inspect: "Die Bahnhofsuhr kann als Zeitreferenz für den Rettungslauf dienen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "karoviertel",
        name: "Karoviertel",
        pct: pct(37.38, 27.78, 14.0, 42.92),
        kind: HotspotKind::Exit,
        look: "Ein Abzweig führt Richtung Karoviertel.",
        inspect: "Die Fallakte nennt dort eine Druckfreigabe.",
        talk_id: None,
    },
    HotspotSpec {
        id: "street_mural",
        name: "Wandbild",
        pct: pct(12.52, 4.28, 20.47, 53.55),
        kind: HotspotKind::Prop,
        look: "Ein Wandbild bedeckt die Ecke.",
        inspect: "Die Farbfläche ist für die aktuelle Greenscreen-Referenz nicht vorgesehen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "corner_kiosk",
        name: "Kiosk",
        pct: pct(58.48, 41.11, 11.0, 20.56),
        kind: HotspotKind::Prop,
        look: "Ein Kiosk steht an der Kreuzung. Die Auslage besteht aus Kaffee, Klebeband und sehr entschlossenen Preisschildern.",
        inspect: "Durch das Seitenfenster hängt eine Rolle Transparentband halb aus dem Spender.",
        talk_id: None,
    },
    HotspotSpec {
        id: "city_notice",
        name: "Drehhinweis",
        pct: pct(32.73, 31.86, 3.05, 12.25),
        kind: HotspotKind::Prop,
        look: "Ein schmaler Aushang klebt am Laternenmast.",
        inspect: "Der Aushang nennt Bahnhofsuhr, Wasserturmlicht und ein reflektierendes Kofferschild am Durchgang.",
        talk_id: None,
    },
];

const STATION_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "schanzenstrasse",
        name: "Zur Schanze",
        pct: pct(6.31, 28.86, 14.0, 40.22),
        kind: HotspotKind::Exit,
        look: "Zurück Richtung Schanzenstraße.",
        inspect: "Die Straße führt zurück zur Kreuzung.",
        talk_id: None,
    },
    HotspotSpec {
        id: "schanzenpark",
        name: "Schanzenpark",
        pct: pct(79.88, 32.63, 15.56, 31.81),
        kind: HotspotKind::Exit,
        look: "Der Weg steigt Richtung Park an.",
        inspect: "Im Park kann die Lichtreferenz für den Greenscreen aufgenommen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "station_clock",
        name: "Bahnhofsuhr",
        pct: pct(41.8, 22.44, 4.19, 12.17),
        kind: HotspotKind::Prop,
        look: "Die Bahnhofsuhr hängt über dem Durchgang. Sie ist das unromantischste Zeitportal Hamburgs.",
        inspect: "Die Uhr liefert eine eindeutige Zeitmarke für die Außenreferenz. Der Host braucht eine Minute, die nicht diskutiert.",
        talk_id: None,
    },
    HotspotSpec {
        id: "platform_sign",
        name: "Bahnsteigschild",
        pct: pct(65.74, 31.95, 7.34, 8.5),
        kind: HotspotKind::Prop,
        look: "Ein Schild weist zum Bahnsteig.",
        inspect:
            "Das Schild bestätigt die Richtung zum Bahnsteig. Für Fall 000 wird nur die Uhr benötigt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "busker_case",
        name: "Straßenmusiker-Koffer",
        pct: pct(31.07, 57.42, 7.21, 17.33),
        kind: HotspotKind::Character,
        look: "Ein offener Koffer liegt vor einem Straßenmusiker. Eine Reflexfolie hält das Spendenschild sichtbar und wirft Regielicht in die falsche Richtung.",
        inspect:
            "Der Musiker steht am Durchgang. Wer seinen Reflektor will, muss erst beweisen, dass diese Rettungsaktion nicht nur eine sehr komplizierte Ausrede ist.",
        talk_id: Some("busker"),
    },
    HotspotSpec {
        id: "public_phone",
        name: "Öffentliches Telefon",
        pct: pct(70.45, 52.89, 2.27, 12.25),
        kind: HotspotKind::Prop,
        look: "Ein öffentliches Telefon hängt am Rand des Durchgangs, als hätte jemand vergessen, ihm mitzuteilen, welches Jahrzehnt gerade ist.",
        inspect: "Der Hörer ist kalt, die Leitung tot. Als Stadtzeit-Anker taugt es weniger als die Bahnhofsuhr.",
        talk_id: None,
    },
];

const PARK_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "sternschanze_station",
        name: "Zur Sternschanze",
        pct: pct(5.54, 53.33, 20.4, 21.03),
        kind: HotspotKind::Exit,
        look: "Zurück zum Bahnhof.",
        inspect: "Der Weg führt zurück zur Sternschanze.",
        talk_id: None,
    },
    HotspotSpec {
        id: "schanzenstrasse",
        name: "Runter zur Straße",
        pct: pct(5.9, 75.11, 13.0, 16.0),
        kind: HotspotKind::Exit,
        look: "Ein Weg zurück ins Viertel.",
        inspect: "Der Weg führt zurück zur Schanzenstraße.",
        talk_id: None,
    },
    HotspotSpec {
        id: "water_tower",
        name: "Wasserturm",
        pct: pct(36.46, 6.61, 10.31, 48.36),
        kind: HotspotKind::Prop,
        look: "Der Wasserturm steht oberhalb des Parks. Sein Backstein hält das Abendlicht wie ein Farbfeld.",
        inspect:
            "Die Backsteinfläche liefert eine klare Lichtreferenz für den Greenscreen-Hintergrund und damit einen zweiten Anker in der Gegenwart.",
        talk_id: None,
    },
    HotspotSpec {
        id: "city_reflector",
        name: "Lichtreflektor",
        pct: pct(64.37, 62.22, 5.53, 13.64),
        kind: HotspotKind::Pickup,
        look: "Ein kleiner Reflektor liegt neben dem Weg. Er sieht mobil aus, bis die Schrauben ihre Meinung äußern.",
        inspect: "Er ist festgeschraubt. Der brauchbare mobile Reflektor hängt am Koffer des Straßenmusikers.",
        talk_id: None,
    },
    HotspotSpec {
        id: "tv_tower_view",
        name: "Fernsehturmblick",
        pct: pct(74.12, 24.45, 2.97, 33.83),
        kind: HotspotKind::Prop,
        look: "Der Fernsehturm ist in der Ferne sichtbar.",
        inspect: "Der Turm gehört nicht zur aktuellen Referenzaufnahme.",
        talk_id: None,
    },
];

const KARO_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "schanzenstrasse",
        name: "Zurück zur Schanze",
        pct: pct(7.71, 14.28, 14.0, 50.22),
        kind: HotspotKind::Exit,
        look: "Zurück Richtung Schanzenstraße.",
        inspect: "Der Rückweg zur Kreuzung ist frei.",
        talk_id: None,
    },
    HotspotSpec {
        id: "print_shop",
        name: "Copyshop",
        pct: pct(31.78, 17.03, 21.01, 47.64),
        kind: HotspotKind::Pickup,
        look: "Im Copyshop liegen Farbfächer, frisch geschnittene Karten und ein Laminiergerät mit zu viel Selbstvertrauen.",
        inspect: "Eine freigegebene Host-Karte liegt am Tresen. Der gleiche Tresen kann ein Kofferschild drucken, wenn der Wortlaut stimmt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "record_store",
        name: "Plattenladen",
        pct: pct(65.62, 29.06, 18.86, 36.5),
        kind: HotspotKind::Prop,
        look: "Ein kleiner Laden mit Plakaten im Fenster.",
        inspect: "Im Fenster steht: Kein Ankauf von Ideen, die nur fast eine Platte sind.",
        talk_id: None,
    },
    HotspotSpec {
        id: "market_boxes",
        name: "Marktkisten",
        pct: pct(16.34, 59.78, 14.0, 16.25),
        kind: HotspotKind::Prop,
        look: "Kisten und Kartons stehen am Rand des Gehwegs. Eine davon wirkt, als hätte sie auf einen Nebenquest gewartet.",
        inspect: "Ein Stück Karton ist stabil genug für ein provisorisches Schild.",
        talk_id: None,
    },
];

const HOSPITAL_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "hospital_monitor",
        name: "Überwachungsmonitor",
        pct: pct(46.4, 31.8, 4.59, 10.25),
        kind: HotspotKind::Prop,
        look: "Der Monitor zeigt stabile Werte. Der Stromschlag war real, aber nicht endgültig.",
        inspect: "Puls, Sauerstoff und EKG sind wieder im grünen Bereich.",
        talk_id: None,
    },
    HotspotSpec {
        id: "call_button",
        name: "Rufknopf",
        pct: pct(43.52, 35.83, 2.31, 9.58),
        kind: HotspotKind::Prop,
        look: "Ein roter Rufknopf hängt am Bett.",
        inspect: "Ohne Pflegerückmeldung kommst du nicht aus dem Zimmer.",
        talk_id: None,
    },
    HotspotSpec {
        id: "nurse_station",
        name: "Pflegekraft",
        pct: pct(67.34, 35.39, 11.68, 36.11),
        kind: HotspotKind::Character,
        look: "Die Pflegekraft prüft Akte und Monitor.",
        inspect: "Sie wartet auf klare Werte und eine sachliche Erklärung.",
        talk_id: Some("nurse"),
    },
    HotspotSpec {
        id: "discharge_clipboard",
        name: "Entlassungsbogen",
        pct: pct(11.04, 69.58, 9.45, 12.0),
        kind: HotspotKind::Pickup,
        look: "Ein Formular liegt auf dem Nachttisch.",
        inspect: "Der Bogen darf erst mit stabilen Werten und Rücksprache mitgenommen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "hospital_exit",
        name: "Korridortür",
        pct: pct(86.98, 0.11, 6.02, 70.81),
        kind: HotspotKind::Prop,
        look: "Die Tür führt in den Krankenhauskorridor.",
        inspect: "Ohne Entlassungsbogen endet der Weg an der Stationsschleuse.",
        talk_id: None,
    },
];

const SPRINKLER_COURTYARD_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "alarm_panel",
        name: "Alarmfeld",
        pct: pct(38.06, 20.83, 4.42, 18.03),
        kind: HotspotKind::Prop,
        look: "Das Alarmfeld protokolliert den Sprinklerlauf.",
        inspect: "Der ausgelöste Kreis muss bestätigt werden, bevor jemand zurück in den Bau darf.",
        talk_id: None,
    },
    HotspotSpec {
        id: "extinguisher_cabinet",
        name: "Löschschrank",
        pct: pct(43.77, 22.74, 6.01, 27.34),
        kind: HotspotKind::Prop,
        look: "Der Schrank ist geöffnet, aber vollständig.",
        inspect: "Es fehlt kein Löscher. Der Schaden ist Wasser, nicht Feuer.",
        talk_id: None,
    },
    HotspotSpec {
        id: "safety_officer",
        name: "Sicherheitsdienst",
        pct: pct(67.4, 21.92, 8.32, 56.31),
        kind: HotspotKind::Character,
        look: "Der Sicherheitsdienst schreibt den Vorfall auf.",
        inspect: "Er lässt dich erst nach technischer Rückmeldung zurück.",
        talk_id: Some("safety"),
    },
    HotspotSpec {
        id: "fire_return_door",
        name: "Rückweg ins Gebäude",
        pct: pct(22.24, 7.89, 13.78, 60.97),
        kind: HotspotKind::Prop,
        look: "Die Tür zurück zum Produktionsgebäude ist nass, aber offen.",
        inspect: "Der Rückweg ist erst nach Alarm- und Materialprüfung frei.",
        talk_id: None,
    },
    HotspotSpec {
        id: "cable_bin",
        name: "Kabelmüll in Tonne",
        pct: pct(0.0, 82.75, 15.76, 17.25),
        kind: HotspotKind::Prop,
        look: "In der Tonne liegt nasser Kabelmüll. Jedes Ende sieht aus, als hätte es eine Erklärung vorbereitet.",
        inspect: "Die Kabelreste sind bereits ausgesondert. Der Sprinkleralarm hängt nicht an ihnen, sondern am bestätigten Alarmkreis.",
        talk_id: None,
    },
];

const PROP_STORAGE_COLLAPSE_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "brace_beam",
        name: "Stützstrebe",
        pct: pct(47.03, 0.0, 13.0, 73.42),
        kind: HotspotKind::Prop,
        look: "Eine Strebe hält die gefallenen Requisiten gerade noch.",
        inspect: "Wenn sie korrekt sitzt, kann der Durchgang freigeräumt werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "release_rope",
        name: "Notleine",
        pct: pct(64.86, 0.0, 5.0, 40.0),
        kind: HotspotKind::Prop,
        look: "Eine Leine hängt an der oberen Traverse.",
        inspect: "Sie bewegt einen Riegel über dem Durchgang.",
        talk_id: None,
    },
    HotspotSpec {
        id: "workshop_gap",
        name: "Freier Spalt",
        pct: pct(25.65, 30.14, 10.37, 42.92),
        kind: HotspotKind::Prop,
        look: "Zwischen den Teilen bleibt ein schmaler Spalt.",
        inspect: "Der Spalt reicht nur, wenn die Strebe hält und der Riegel offen ist.",
        talk_id: None,
    },
    HotspotSpec {
        id: "collapse_exit",
        name: "Werkstatttür",
        pct: pct(74.72, 10.5, 9.29, 61.31),
        kind: HotspotKind::Prop,
        look: "Die Tür führt zurück in den Szenenbau.",
        inspect: "Der direkte Weg ist von Kulissenteilen blockiert.",
        talk_id: None,
    },
    HotspotSpec {
        id: "paint_cans",
        name: "Farbdosen",
        pct: pct(1.9, 76.22, 16.95, 16.0),
        kind: HotspotKind::Prop,
        look: "Farbdosen stapeln sich am Boden. Die Etiketten versprechen Bühnennebel, Betonoptik und Entscheidungen, die niemand dokumentiert hat.",
        inspect: "Die Dosen sind zu schwer für schnelle Improvisation und zu nass für saubere Ausreden. Der Fluchtweg braucht Strebe und Notleine, nicht Farbe.",
        talk_id: None,
    },
];

const ARCHIVE_RECOVERY_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "recovery_terminal",
        name: "Rettungsterminal",
        pct: pct(49.46, 12.03, 22.78, 43.7),
        kind: HotspotKind::Prop,
        look: "Das Terminal zeigt die letzte lesbare Frame-ID.",
        inspect: "Die Session muss mit Prüfsumme abgeschlossen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "checksum_printout",
        name: "Prüfsummenzettel",
        pct: pct(67.68, 54.69, 13.56, 19.7),
        kind: HotspotKind::Pickup,
        look: "Ein Ausdruck liegt neben der Tastatur.",
        inspect: "Die Prüfsumme passt zum beschädigten Videoframe.",
        talk_id: None,
    },
    HotspotSpec {
        id: "archive_drive",
        name: "Archivlaufwerk",
        pct: pct(0.87, 34.11, 12.0, 32.84),
        kind: HotspotKind::Prop,
        look: "Das Laufwerk klackert in kurzen, gleichmäßigen Abständen.",
        inspect: "Die Medien sind lesbar, aber die Session ist noch nicht quittiert.",
        talk_id: None,
    },
    HotspotSpec {
        id: "control_room_return",
        name: "Tür zur Regie",
        pct: pct(27.84, 0.0, 12.46, 63.11),
        kind: HotspotKind::Prop,
        look: "Die Tür führt zurück zur Regie.",
        inspect: "Der Rückweg ist erst nach einer sauber abgeschlossenen Rettungssession frei.",
        talk_id: None,
    },
];

const HOTSPOT_POLYGONS: &[HotspotPolygonSpec] = &[
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "office_hall", points: &[(5.43, 25.47), (11.52, 25.47), (11.52, 68.39), (5.43, 68.39)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "control_room", points: &[(94.43, 26.61), (99.28, 26.61), (99.28, 78.44), (94.35, 74.9)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "greenscreen_wall", points: &[(38.65, 16.75), (71.41, 16.75), (71.41, 63.97), (38.65, 63.97)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "floor_marks", points: &[(38.87, 63.25), (60.87, 63.25), (60.87, 72.25), (38.87, 72.25)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "klixx_table", points: &[(23.54, 51.11), (40.22, 51.11), (40.22, 75.5), (23.54, 75.5)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "chat_preview", points: &[(19.94, 29.56), (24.51, 29.56), (24.51, 47.92), (19.94, 47.92)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "camera_one", points: &[(62.34, 41.44), (73.09, 41.44), (73.09, 76.13), (62.34, 76.13)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "mentor_shadow", points: &[(75.74, 37.92), (80.41, 37.92), (80.41, 72.17), (75.74, 72.17)] },
    HotspotPolygonSpec { scene_id: "greenscreen_studio", hotspot_id: "greenscreen_setup", points: &[(49.18, 35.89), (72.18, 35.89), (72.18, 72.89), (49.18, 72.89)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "building_courtyard", points: &[(2.49, 21.81), (9.07, 21.81), (9.07, 68.67), (2.49, 68.67)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "greenscreen_studio", points: &[(16.33, 25.78), (23.25, 25.78), (23.25, 67.45), (16.33, 67.45)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "control_room", points: &[(34.16, 26.25), (40.41, 26.25), (40.41, 67.14), (34.16, 67.14)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "server_room", points: &[(84.65, 27.42), (89.7, 27.42), (89.7, 67.98), (84.65, 67.98)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "schanzenstrasse", points: &[(73.18, 27.39), (79.1, 27.39), (79.1, 64.7), (73.18, 64.7)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "dispo_board", points: &[(24.53, 31.94), (32.55, 31.94), (32.55, 54.61), (24.53, 54.61)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "equipment_storage", points: &[(43.54, 26.25), (56.54, 26.25), (56.54, 67.08), (43.54, 67.08)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "staircase", points: &[(60.8, 13.11), (69.78, 13.11), (69.78, 63.17), (60.8, 63.17)] },
    HotspotPolygonSpec { scene_id: "office_hall", hotspot_id: "route_map", points: &[(92.8, 31.69), (99.31, 31.69), (99.31, 53.33), (92.8, 53.33)] },
    HotspotPolygonSpec { scene_id: "building_courtyard", hotspot_id: "office_hall", points: &[(50.44, 31.58), (57.89, 31.58), (57.89, 63.36), (50.44, 63.36)] },
    HotspotPolygonSpec { scene_id: "building_courtyard", hotspot_id: "set_workshop", points: &[(80.72, 33.5), (85.89, 33.5), (85.89, 67.42), (80.72, 67.42)] },
    HotspotPolygonSpec { scene_id: "building_courtyard", hotspot_id: "schanzenstrasse", points: &[(7.67, 25.97), (15.42, 25.97), (15.42, 73.3), (7.67, 73.3)] },
    HotspotPolygonSpec { scene_id: "building_courtyard", hotspot_id: "loading_zone", points: &[(19.56, 46.33), (37.56, 46.33), (37.56, 72.02), (19.56, 72.02)] },
    HotspotPolygonSpec { scene_id: "building_courtyard", hotspot_id: "address_plate", points: &[(68.55, 30.78), (72.59, 30.78), (72.59, 43.59), (68.55, 43.59)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "office_hall", points: &[(3.39, 23.92), (10.72, 23.92), (10.72, 68.56), (3.39, 68.56)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "greenscreen_studio", points: &[(22.16, 15.14), (46.91, 15.14), (46.91, 44.03), (22.16, 44.03)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "rehearsal_monitor", points: &[(32.57, 46.47), (36.03, 46.47), (36.03, 53.78), (32.57, 53.78)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "graphics_terminal", points: &[(66.11, 42.47), (78.75, 42.47), (78.75, 69.28), (66.11, 69.28)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "intercom_voice", points: &[(56.29, 43.81), (63.19, 43.81), (63.19, 60.34), (56.29, 60.34)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "on_air_lamp", points: &[(87.66, 10.25), (92.0, 10.25), (92.0, 16.25), (87.66, 16.25)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "video_schimmelbrueder", points: &[(63.58, 33.36), (67.48, 33.36), (67.48, 41.3), (63.58, 41.3)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "video_icemachine", points: &[(68.2, 33.36), (72.1, 33.36), (72.1, 41.3), (68.2, 41.3)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "video_brassband", points: &[(72.82, 33.36), (76.72, 33.36), (76.72, 41.3), (72.82, 41.3)] },
    HotspotPolygonSpec { scene_id: "control_room", hotspot_id: "mixing_console", points: &[(82.71, 43.58), (99.82, 43.58), (99.82, 65.97), (82.71, 65.97)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "control_room", points: &[(1.32, 21.94), (7.94, 21.94), (7.94, 38.19), (1.32, 38.19)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "casting_table", points: &[(26.61, 21.7), (40.11, 24.06), (16.56, 55.31), (8.87, 54.34), (9.14, 31.56)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "mold_rack", points: &[(50.38, 18.37), (57.15, 19.48), (40.67, 66.49), (19.61, 66.49)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "patterned_belt", points: &[(61.56, 12.12), (72.2, 11.56), (85.81, 57.67), (54.57, 58.23)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "mold_token", points: &[(41.3, 80.56), (50.27, 80.56), (50.27, 93.78), (41.3, 93.78)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "song_exit_gate", points: &[(86.48, 29.36), (99.83, 29.36), (99.83, 82.47), (86.48, 82.47)] },
    HotspotPolygonSpec { scene_id: "video_schimmelbrueder", hotspot_id: "factory_floor", points: &[(7.57, 66.0), (86.81, 66.0), (86.81, 100.0), (7.57, 100.0)] },
    HotspotPolygonSpec { scene_id: "video_icemachine", hotspot_id: "control_room", points: &[(3.31, 17.11), (9.98, 17.11), (9.98, 63.33), (3.31, 63.33)] },
    HotspotPolygonSpec { scene_id: "video_icemachine", hotspot_id: "ice_machine", points: &[(51.02, 8.53), (70.42, 8.53), (70.42, 72.84), (51.02, 72.84)] },
    HotspotPolygonSpec { scene_id: "video_icemachine", hotspot_id: "cleaning_bucket", points: &[(11.84, 49.22), (24.32, 49.22), (24.32, 77.28), (11.84, 77.28)] },
    HotspotPolygonSpec { scene_id: "video_icemachine", hotspot_id: "service_counter", points: &[(64.38, 39.83), (100.0, 39.83), (100.0, 57.72), (64.38, 57.72)] },
    HotspotPolygonSpec { scene_id: "video_brassband", hotspot_id: "control_room", points: &[(4.17, 19.59), (12.02, 19.59), (12.02, 64.67), (4.17, 64.67)] },
    HotspotPolygonSpec { scene_id: "video_brassband", hotspot_id: "brass_players", points: &[(40.48, 34.41), (44.41, 31.08), (48.6, 45.94), (45.43, 50.24), (45.32, 65.94), (50.11, 66.35), (50.7, 47.88), (48.82, 36.22), (53.28, 33.02), (55.7, 43.16), (57.58, 65.1), (62.69, 63.58), (63.49, 46.35), (68.55, 31.77), (81.13, 34.27), (81.67, 65.8), (74.19, 70.1), (43.76, 71.08), (39.03, 64.69)] },
    HotspotPolygonSpec { scene_id: "video_brassband", hotspot_id: "tuba_bell", points: &[(57.84, 25.61), (63.64, 25.61), (63.64, 47.14), (57.84, 47.14)] },
    HotspotPolygonSpec { scene_id: "video_brassband", hotspot_id: "music_stand", points: &[(45.48, 46.92), (49.99, 46.92), (49.99, 67.25), (45.48, 67.25)] },
    HotspotPolygonSpec { scene_id: "video_brassband", hotspot_id: "stage_floor", points: &[(4.27, 79.03), (76.27, 79.03), (76.27, 97.03), (4.27, 97.03)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "office_hall", points: &[(4.35, 25.69), (9.86, 25.69), (9.86, 71.58), (4.35, 71.58)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "server_racks", points: &[(17.17, 19.3), (39.04, 19.3), (39.04, 72.16), (17.17, 72.16)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "video_hub", points: &[(41.82, 21.31), (61.16, 21.31), (61.16, 70.98), (41.82, 70.98)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "sdi_spool", points: &[(70.59, 51.56), (83.74, 51.56), (83.74, 75.53), (70.59, 75.53)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "sdi_label_printer", points: &[(62.8, 41.41), (68.0, 41.41), (68.0, 50.94), (62.8, 50.94)] },
    HotspotPolygonSpec { scene_id: "server_room", hotspot_id: "cable_loop", points: &[(66.53, 26.92), (69.93, 26.92), (69.93, 42.64), (66.53, 42.64)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "building_courtyard", points: &[(4.19, 27.92), (9.21, 27.92), (9.21, 72.7), (4.19, 72.7)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "set_pieces", points: &[(14.65, 13.39), (38.65, 13.39), (38.65, 70.28), (14.65, 70.28)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "gaffer_roll", points: &[(57.0, 51.33), (60.27, 51.33), (60.27, 57.5), (57.0, 57.5)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "bike_workshop_trace", points: &[(67.49, 13.28), (83.49, 13.28), (83.49, 35.28), (67.49, 35.28)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "studio_door_15", points: &[(85.53, 29.31), (93.01, 29.31), (93.01, 67.28), (85.53, 67.28)] },
    HotspotPolygonSpec { scene_id: "set_workshop", hotspot_id: "brushes", points: &[(41.74, 46.5), (52.18, 46.5), (52.18, 56.39), (41.74, 56.39)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "building_courtyard", points: &[(3.12, 38.39), (10.1, 38.39), (10.1, 70.89), (3.12, 70.89)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "sternschanze_station", points: &[(73.14, 31.94), (100.0, 31.94), (100.0, 71.11), (73.14, 71.11)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "karoviertel", points: &[(37.38, 27.78), (51.38, 27.78), (51.38, 70.7), (37.38, 70.7)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "street_mural", points: &[(12.52, 4.28), (32.99, 4.28), (32.99, 57.83), (12.52, 57.83)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "corner_kiosk", points: &[(58.48, 41.11), (69.48, 41.11), (69.48, 61.67), (58.48, 61.67)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "city_notice", points: &[(32.73, 31.86), (35.78, 31.86), (35.78, 44.11), (32.73, 44.11)] },
    HotspotPolygonSpec { scene_id: "schanzenstrasse", hotspot_id: "bicycle", points: &[(21.9, 57.34), (31.53, 57.34), (31.53, 71.67), (21.9, 71.67)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "schanzenstrasse", points: &[(6.31, 28.86), (20.31, 28.86), (20.31, 69.08), (6.31, 69.08)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "schanzenpark", points: &[(79.88, 32.63), (95.44, 32.63), (95.44, 64.44), (79.88, 64.44)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "station_clock", points: &[(41.8, 22.44), (45.99, 22.44), (45.99, 34.61), (41.8, 34.61)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "platform_sign", points: &[(65.74, 31.95), (73.08, 31.95), (73.08, 40.45), (65.74, 40.45)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "busker_case", points: &[(31.07, 57.42), (38.28, 57.42), (38.28, 74.75), (31.07, 74.75)] },
    HotspotPolygonSpec { scene_id: "sternschanze_station", hotspot_id: "public_phone", points: &[(70.45, 52.89), (72.72, 52.89), (72.72, 65.14), (70.45, 65.14)] },
    HotspotPolygonSpec { scene_id: "schanzenpark", hotspot_id: "sternschanze_station", points: &[(5.54, 53.33), (25.94, 53.33), (25.94, 74.36), (5.54, 74.36)] },
    HotspotPolygonSpec { scene_id: "schanzenpark", hotspot_id: "schanzenstrasse", points: &[(5.9, 75.11), (18.9, 75.11), (18.9, 91.11), (5.9, 91.11)] },
    HotspotPolygonSpec { scene_id: "schanzenpark", hotspot_id: "water_tower", points: &[(36.46, 6.61), (46.77, 6.61), (46.77, 54.97), (36.46, 54.97)] },
    HotspotPolygonSpec { scene_id: "schanzenpark", hotspot_id: "city_reflector", points: &[(64.37, 62.22), (69.9, 62.22), (69.9, 75.86), (64.37, 75.86)] },
    HotspotPolygonSpec { scene_id: "schanzenpark", hotspot_id: "tv_tower_view", points: &[(74.12, 24.45), (77.09, 24.45), (77.09, 58.28), (74.12, 58.28)] },
    HotspotPolygonSpec { scene_id: "karoviertel", hotspot_id: "schanzenstrasse", points: &[(7.71, 14.28), (21.71, 14.28), (21.71, 64.5), (7.71, 64.5)] },
    HotspotPolygonSpec { scene_id: "karoviertel", hotspot_id: "print_shop", points: &[(31.78, 17.03), (52.79, 17.03), (52.79, 64.67), (31.78, 64.67)] },
    HotspotPolygonSpec { scene_id: "karoviertel", hotspot_id: "record_store", points: &[(65.62, 29.06), (84.48, 29.06), (84.48, 65.56), (65.62, 65.56)] },
    HotspotPolygonSpec { scene_id: "karoviertel", hotspot_id: "market_boxes", points: &[(16.34, 59.78), (30.34, 59.78), (30.34, 76.03), (16.34, 76.03)] },
    HotspotPolygonSpec { scene_id: "hospital_room", hotspot_id: "hospital_monitor", points: &[(46.4, 31.8), (50.99, 31.8), (50.99, 42.05), (46.4, 42.05)] },
    HotspotPolygonSpec { scene_id: "hospital_room", hotspot_id: "call_button", points: &[(43.52, 35.83), (45.83, 35.83), (45.83, 45.41), (43.52, 45.41)] },
    HotspotPolygonSpec { scene_id: "hospital_room", hotspot_id: "nurse_station", points: &[(67.34, 35.39), (79.02, 35.39), (79.02, 71.5), (67.34, 71.5)] },
    HotspotPolygonSpec { scene_id: "hospital_room", hotspot_id: "discharge_clipboard", points: &[(11.04, 69.58), (20.49, 69.58), (20.49, 81.58), (11.04, 81.58)] },
    HotspotPolygonSpec { scene_id: "hospital_room", hotspot_id: "hospital_exit", points: &[(86.98, 0.11), (93.0, 0.11), (93.0, 70.92), (86.98, 70.92)] },
    HotspotPolygonSpec { scene_id: "sprinkler_courtyard", hotspot_id: "alarm_panel", points: &[(38.06, 20.83), (42.48, 20.83), (42.48, 38.86), (38.06, 38.86)] },
    HotspotPolygonSpec { scene_id: "sprinkler_courtyard", hotspot_id: "extinguisher_cabinet", points: &[(43.77, 22.74), (49.78, 22.74), (49.78, 50.08), (43.77, 50.08)] },
    HotspotPolygonSpec { scene_id: "sprinkler_courtyard", hotspot_id: "safety_officer", points: &[(67.4, 21.92), (75.72, 21.92), (75.72, 78.23), (67.4, 78.23)] },
    HotspotPolygonSpec { scene_id: "sprinkler_courtyard", hotspot_id: "fire_return_door", points: &[(22.24, 7.89), (36.02, 7.89), (36.02, 68.86), (22.24, 68.86)] },
    HotspotPolygonSpec { scene_id: "sprinkler_courtyard", hotspot_id: "cable_bin", points: &[(0.0, 82.75), (15.76, 82.75), (15.76, 100.0), (0.0, 100.0)] },
    HotspotPolygonSpec { scene_id: "prop_storage_collapse", hotspot_id: "brace_beam", points: &[(47.03, 0.0), (60.03, 0.0), (60.03, 73.42), (47.03, 73.42)] },
    HotspotPolygonSpec { scene_id: "prop_storage_collapse", hotspot_id: "release_rope", points: &[(64.86, 0.0), (69.86, 0.0), (69.86, 40.0), (64.86, 40.0)] },
    HotspotPolygonSpec { scene_id: "prop_storage_collapse", hotspot_id: "workshop_gap", points: &[(25.81, 35.52), (28.28, 43.44), (35.38, 71.63), (31.45, 69.83)] },
    HotspotPolygonSpec { scene_id: "prop_storage_collapse", hotspot_id: "collapse_exit", points: &[(74.72, 10.5), (84.01, 10.5), (84.01, 71.81), (74.72, 71.81)] },
    HotspotPolygonSpec { scene_id: "prop_storage_collapse", hotspot_id: "paint_cans", points: &[(1.9, 76.22), (18.85, 76.22), (18.85, 92.22), (1.9, 92.22)] },
    HotspotPolygonSpec { scene_id: "archive_recovery", hotspot_id: "archive_drive", points: &[(0.87, 34.11), (12.87, 34.11), (12.87, 66.95), (0.87, 66.95)] },
    HotspotPolygonSpec { scene_id: "archive_recovery", hotspot_id: "recovery_terminal", points: &[(49.46, 12.03), (72.24, 12.03), (72.24, 55.73), (49.46, 55.73)] },
    HotspotPolygonSpec { scene_id: "archive_recovery", hotspot_id: "checksum_printout", points: &[(67.68, 54.69), (81.24, 54.69), (81.24, 74.39), (67.68, 74.39)] },
    HotspotPolygonSpec { scene_id: "archive_recovery", hotspot_id: "control_room_return", points: &[(27.84, 0.0), (40.3, 0.0), (40.3, 63.11), (27.84, 63.11)] },
];

const STUDIO_WALKABLE: &[(f32, f32)] = &[
    (16.13, 65.73),
    (28.17, 76.7),
    (41.61, 65.59),
    (80.0, 62.0),
    (100.0, 87.95),
    (100.0, 100.0),
    (0.0, 100.0),
    (0.0, 76.15),
    (5.91, 69.76),
];
const OFFICE_HALL_WALKABLE: &[(f32, f32)] = &[
    (11.45, 67.12),
    (91.61, 69.48),
    (100.0, 73.65),
    (100.0, 100.0),
    (0.0, 100.0),
    (0.11, 76.56),
    (2.96, 74.62),
    (3.17, 72.67),
];
const COURTYARD_WALKABLE: &[(f32, f32)] = &[
    (7.74, 76.7),
    (19.73, 73.37),
    (31.83, 74.62),
    (41.51, 72.53),
    (62.26, 73.51),
    (75.27, 70.73),
    (82.85, 77.95),
    (86.51, 76.28),
    (93.55, 81.15),
    (99.62, 78.65),
    (100.0, 86.15),
    (91.08, 85.87),
    (89.09, 82.67),
    (87.2, 87.4),
    (84.41, 90.17),
    (81.13, 94.2),
    (71.45, 100.0),
    (16.67, 99.62),
    (15.32, 95.45),
    (10.22, 92.26),
];
const CONTROL_ROOM_WALKABLE: &[(f32, f32)] = &[
    (0.0, 69.9),
    (21.18, 66.42),
    (100.0, 89.2),
    (100.0, 100.0),
    (0.0, 100.0),
];
const VIDEO_SCHIMMEL_WALKABLE: &[(f32, f32)] = &[
    (8.66, 64.76),
    (40.16, 66.56),
    (56.56, 20.45),
    (59.62, 28.78),
    (53.71, 60.59),
    (80.05, 60.45),
    (84.41, 72.67),
    (89.57, 70.17),
    (100.0, 84.76),
    (100.0, 100.0),
    (0.0, 100.0),
    (0.0, 73.78),
];
const VIDEO_ICEMACHINE_WALKABLE: &[(f32, f32)] = &[
    (11.29, 63.3),
    (12.26, 77.19),
    (24.78, 77.6),
    (25.32, 60.52),
    (47.58, 66.08),
    (64.14, 78.02),
    (100.0, 97.88),
    (100.0, 100.0),
    (0.0, 100.0),
    (0.0, 72.88),
];
const VIDEO_BRASSBAND_WALKABLE: &[(f32, f32)] = &[
    (0.0, 73.58),
    (15.16, 65.24),
    (26.77, 62.05),
    (33.23, 59.97),
    (38.92, 59.83),
    (39.52, 67.05),
    (59.25, 66.77),
    (75.48, 69.97),
    (77.15, 100.0),
    (0.0, 100.0),
];
const SERVER_ROOM_WALKABLE: &[(f32, f32)] = &[
    (0.0, 77.53),
    (9.46, 71.28),
    (18.01, 73.51),
    (39.14, 74.06),
    (61.13, 73.37),
    (69.89, 73.65),
    (72.58, 78.23),
    (75.54, 78.51),
    (79.62, 78.51),
    (83.76, 78.37),
    (84.78, 75.59),
    (84.3, 68.65),
    (87.26, 68.37),
    (93.98, 73.37),
    (100.0, 77.67),
    (100.0, 100.0),
    (0.0, 100.0),
];
const SET_WORKSHOP_WALKABLE: &[(f32, f32)] = &[
    (0.0, 77.26),
    (11.29, 71.7),
    (39.78, 72.12),
    (65.43, 72.4),
    (67.47, 67.95),
    (100.0, 69.34),
    (100.0, 100.0),
    (0.0, 100.0),
];
const SCHANZENSTRASSE_WALKABLE: &[(f32, f32)] = &[
    (0.05, 71.15),
    (37.47, 72.4),
    (58.23, 75.59),
    (72.9, 74.34),
    (73.44, 60.59),
    (92.63, 61.98),
    (100.0, 72.4),
    (86.24, 73.78),
    (86.02, 90.31),
    (92.85, 100.0),
    (0.0, 100.0),
];
const STATION_WALKABLE: &[(f32, f32)] = &[
    (11.51, 91.7),
    (11.02, 80.87),
    (9.46, 70.87),
    (21.94, 70.03),
    (25.81, 72.4),
    (32.1, 76.15),
    (38.82, 75.03),
    (40.22, 68.65),
    (54.52, 67.4),
    (65.59, 63.92),
    (77.9, 65.87),
    (95.81, 66.84),
    (100.0, 72.67),
    (100.0, 100.0),
    (0.0, 100.0),
];
const PARK_WALKABLE: &[(f32, f32)] = &[
    (0.0, 91.98),
    (7.42, 91.98),
    (12.31, 79.2),
    (15.0, 79.34),
    (17.74, 90.03),
    (22.58, 90.45),
    (29.09, 83.23),
    (35.65, 78.09),
    (33.92, 59.62),
    (40.59, 56.7),
    (48.66, 55.45),
    (56.02, 56.7),
    (59.52, 60.59),
    (61.02, 66.98),
    (60.65, 73.78),
    (68.82, 78.09),
    (72.04, 77.53),
    (74.78, 75.87),
    (92.15, 76.84),
    (91.02, 87.12),
    (100.0, 100.0),
    (0.0, 100.0),
];
const KARO_WALKABLE: &[(f32, f32)] = &[
    (0.0, 67.53),
    (7.63, 65.59),
    (15.0, 65.45),
    (15.43, 72.4),
    (17.1, 76.98),
    (30.75, 76.42),
    (31.88, 66.28),
    (88.06, 66.42),
    (90.86, 73.09),
    (95.48, 76.98),
    (100.0, 74.48),
    (100.0, 100.0),
    (0.0, 100.0),
];
const HOSPITAL_WALKABLE: &[(f32, f32)] = &[
    (31.02, 92.4),
    (40.27, 75.59),
    (58.6, 82.53),
    (65.0, 71.84),
    (91.72, 73.37),
    (95.11, 82.26),
    (100.0, 86.15),
    (100.0, 100.0),
    (27.04, 100.0),
];
const SPRINKLER_COURTYARD_WALKABLE: &[(f32, f32)] = &[
    (22.1, 73.23),
    (39.62, 68.78),
    (65.86, 59.76),
    (76.18, 61.84),
    (76.13, 72.95),
    (72.42, 76.98),
    (71.88, 86.42),
    (85.75, 100.0),
    (17.15, 100.0),
];
const PROP_STORAGE_COLLAPSE_WALKABLE: &[(f32, f32)] = &[
    (56.94, 90.17),
    (67.0, 78.0),
    (72.47, 72.67),
    (83.92, 72.4),
    (89.73, 87.12),
    (82.63, 87.81),
    (81.94, 100.0),
    (45.38, 100.0),
];
const ARCHIVE_RECOVERY_WALKABLE: &[(f32, f32)] = &[
    (16.45, 82.53),
    (25.59, 62.95),
    (40.32, 63.23),
    (44.57, 90.17),
    (53.23, 94.48),
    (54.14, 100.0),
    (14.19, 100.0),
];

const SCENES: &[SceneMeta] = &[
    SceneMeta {
        id: "greenscreen_studio",
        name: "Greenscreen-Studio",
        zone: "Heinrichstraße 9",
        token: (45.0, 83.0),
        walkable: STUDIO_WALKABLE,
        hotspots: STUDIO_HOTSPOTS,
    },
    SceneMeta {
        id: "office_hall",
        name: "Büroflur",
        zone: "Heinrichstraße 9",
        token: (48.0, 82.0),
        walkable: OFFICE_HALL_WALKABLE,
        hotspots: OFFICE_HALL_HOTSPOTS,
    },
    SceneMeta {
        id: "building_courtyard",
        name: "Heinrichstraße-Hof",
        zone: "Produktionsgelände",
        token: (48.0, 82.0),
        walkable: COURTYARD_WALKABLE,
        hotspots: COURTYARD_HOTSPOTS,
    },
    SceneMeta {
        id: "control_room",
        name: "Regie",
        zone: "Heinrichstraße 9",
        token: (22.0, 82.0),
        walkable: CONTROL_ROOM_WALKABLE,
        hotspots: CONTROL_ROOM_HOTSPOTS,
    },
    SceneMeta {
        id: "video_schimmelbrueder",
        name: "Schimmelbrüder-Video",
        zone: "Videoarchiv",
        token: (24.0, 84.0),
        walkable: VIDEO_SCHIMMEL_WALKABLE,
        hotspots: VIDEO_SCHIMMEL_HOTSPOTS,
    },
    SceneMeta {
        id: "video_icemachine",
        name: "Eismaschinen-Video",
        zone: "Videoarchiv",
        token: (50.0, 84.0),
        walkable: VIDEO_ICEMACHINE_WALKABLE,
        hotspots: VIDEO_ICEMACHINE_HOTSPOTS,
    },
    SceneMeta {
        id: "video_brassband",
        name: "Band-Video",
        zone: "Videoarchiv",
        token: (50.0, 84.0),
        walkable: VIDEO_BRASSBAND_WALKABLE,
        hotspots: VIDEO_BRASSBAND_HOTSPOTS,
    },
    SceneMeta {
        id: "server_room",
        name: "Serverraum 11",
        zone: "Heinrichstraße 11",
        token: (22.0, 82.0),
        walkable: SERVER_ROOM_WALKABLE,
        hotspots: SERVER_ROOM_HOTSPOTS,
    },
    SceneMeta {
        id: "set_workshop",
        name: "Haus-15-Szenenbau",
        zone: "Heinrichstraße 15",
        token: (22.0, 82.0),
        walkable: SET_WORKSHOP_WALKABLE,
        hotspots: SET_WORKSHOP_HOTSPOTS,
    },
    SceneMeta {
        id: "schanzenstrasse",
        name: "Schanzenstraße",
        zone: "Hamburg",
        token: (30.0, 82.0),
        walkable: SCHANZENSTRASSE_WALKABLE,
        hotspots: SCHANZENSTRASSE_HOTSPOTS,
    },
    SceneMeta {
        id: "sternschanze_station",
        name: "Bahnhof Sternschanze",
        zone: "Hamburg",
        token: (28.0, 82.0),
        walkable: STATION_WALKABLE,
        hotspots: STATION_HOTSPOTS,
    },
    SceneMeta {
        id: "schanzenpark",
        name: "Schanzenpark",
        zone: "Hamburg",
        token: (24.0, 82.0),
        walkable: PARK_WALKABLE,
        hotspots: PARK_HOTSPOTS,
    },
    SceneMeta {
        id: "karoviertel",
        name: "Karoviertel",
        zone: "Hamburg",
        token: (24.0, 82.0),
        walkable: KARO_WALKABLE,
        hotspots: KARO_HOTSPOTS,
    },
    SceneMeta {
        id: "hospital_room",
        name: "Krankenhauszimmer",
        zone: "Notaufnahme",
        token: (38.0, 82.0),
        walkable: HOSPITAL_WALKABLE,
        hotspots: HOSPITAL_HOTSPOTS,
    },
    SceneMeta {
        id: "sprinkler_courtyard",
        name: "Evakuierungsbereich",
        zone: "Produktionsgelände",
        token: (28.0, 82.0),
        walkable: SPRINKLER_COURTYARD_WALKABLE,
        hotspots: SPRINKLER_COURTYARD_HOTSPOTS,
    },
    SceneMeta {
        id: "prop_storage_collapse",
        name: "Blockierter Szenenbau",
        zone: "Heinrichstraße 15",
        token: (24.0, 84.0),
        walkable: PROP_STORAGE_COLLAPSE_WALKABLE,
        hotspots: PROP_STORAGE_COLLAPSE_HOTSPOTS,
    },
    SceneMeta {
        id: "archive_recovery",
        name: "Archiv-Rettungsplatz",
        zone: "Videoarchiv",
        token: (24.0, 82.0),
        walkable: ARCHIVE_RECOVERY_WALKABLE,
        hotspots: ARCHIVE_RECOVERY_HOTSPOTS,
    },
];

const SCENE_TEXTURE_VARIANTS: &[(&str, &str)] = &[
    (
        "hospital_room_nurse",
        "assets/scenes/hospital_room_nurse.png",
    ),
    (
        "hospital_room_no_clipboard",
        "assets/scenes/hospital_room_no_clipboard.png",
    ),
    (
        "archive_recovery_no_checksum",
        "assets/scenes/archive_recovery_no_checksum.png",
    ),
    (
        "video_schimmelbrueder_no_token",
        "assets/scenes/video_schimmelbrueder_no_token.png",
    ),
];

const ITEMS: &[ItemMeta] = &[
    ItemMeta {
        id: "call_sheet",
        name: "Fallakte 000",
        short: "AKTE",
        description: "Die Akte zum verschwundenen Host: Studio, Regie, Stadtzeit und Schimmelbrüder-Frame.",
    },
    ItemMeta {
        id: "gaffer_tape",
        name: "Gaffer-Tape",
        short: "TAPE",
        description: "Eine Rolle für neue Bodenmarken am Rückholpunkt.",
    },
    ItemMeta {
        id: "sdi_label",
        name: "SDI-Label",
        short: "SDI",
        description: "Ein Etikett für die benannte Signalroute von Haus 11 ins Studio.",
    },
    ItemMeta {
        id: "lower_third_card",
        name: "Host-Karte",
        short: "GRAF",
        description: "Farbkarte und Textfreigabe für den Host-Namen in der Regie.",
    },
    ItemMeta {
        id: "city_reflector",
        name: "Kofferreflektor",
        short: "LICH",
        description: "Der geliehene Reflektor vom Straßenmusiker. Er trägt genug Stadtlicht für den Rückholpunkt.",
    },
    ItemMeta {
        id: "cardboard_piece",
        name: "Pappstück",
        short: "PAPP",
        description: "Ein festes Stück Karton aus den Marktkisten.",
    },
    ItemMeta {
        id: "clear_tape",
        name: "Transparentband",
        short: "BAND",
        description: "Klebeband vom Kiosk, offiziell zu kurz für jede sinnvolle Beschilderung.",
    },
    ItemMeta {
        id: "busker_sign",
        name: "Kofferschild",
        short: "SCHD",
        description: "Ein laminiertes Schild für den Straßenmusiker-Koffer und die kleinste legale Bestechung dieses Falls.",
    },
    ItemMeta {
        id: "mold_token",
        name: "Frame-Probe",
        short: "FORM",
        description:
            "Eine Probe aus der Schimmelbrüder-Halle. Sie gehört zur Musterfolge und hält ein Host-Echo.",
    },
    ItemMeta {
        id: "medical_release",
        name: "Entlassungsbogen",
        short: "ENTL",
        description: "Die Freigabe der Station nach dem Stromschlag.",
    },
    ItemMeta {
        id: "checksum_note",
        name: "Prüfsummenzettel",
        short: "CHK",
        description: "Ein Ausdruck mit Prüfsumme für die Video-Rettungssession.",
    },
];

const MENTOR_CHOICES: &[DialogueChoice] = &[
    DialogueChoice {
        label: "Was ist Fall 000?",
        response: "Ein Host hat einen Archivframe geöffnet und ist nicht zurückgekommen. Wir stabilisieren erst die Gegenwart: Fallakte, Bodenmarken, SDI-Route, Host-Karte, Stadtzeit, Parklicht und der Schimmelbrüder-Frame.",
        flag: "mentor_goal_hint",
        log: "Die Aufnahmeleitung definiert Fall 000: Studio markieren, Signal benennen, Host-Karte laden, Stadt referenzieren, Archivframe stabilisieren.",
    },
    DialogueChoice {
        label: "Warum raus in die Stadt?",
        response: "Weil ein Studio ohne Außenzeit nur behauptet, Gegenwart zu sein. Die Bahnhofsuhr liefert den Takt, der Wasserturm das Licht, und der Reflektor bringt beides zurück ins Bild.",
        flag: "mentor_city_hint",
        log: "Die Stadt liefert Zeit, Licht und einen mobilen Reflektor als Gegenwartsanker.",
    },
    DialogueChoice {
        label: "Warum ich?",
        response: "Weil du noch nicht in der Aufnahme vorkommst. Der Frame kann dich schlechter einsortieren, und das ist heute ausnahmsweise eine Qualifikation.",
        flag: "mentor_intern_hint",
        log: "Der Praktikant übernimmt die Laufwege, weil der geöffnete Frame ihn noch nicht kennt.",
    },
    DialogueChoice {
        label: "Wie starte ich die Rettung?",
        response: "Wenn alle Anker sitzen, starte das Setup im Greenscreen-Studio. Wenn nicht, füttern wir den Host mit einem halben Rückweg und hoffen, dass die Physik Humor hat.",
        flag: "mentor_rehearsal_hint",
        log: "Der Rettungslauf wird am Setup im Greenscreen-Studio gestartet.",
    },
];

const BUSKER_CHOICES: &[DialogueChoice] = &[
    DialogueChoice {
        label: "Welche Zeitreferenz passt?",
        response: "Nimm die Bahnhofsuhr über dem Durchgang. Sie ist im Bild klar lesbar, beleidigend pünktlich und lässt sich nicht vom Chat in Panik nullen.",
        flag: "busker_timing_hint",
        log: "Der Musiker verweist auf die Bahnhofsuhr als harte Zeitreferenz.",
    },
    DialogueChoice {
        label: "Kennst du den Weg zum Park?",
        response: "Geh vom Bahnhof bergauf zum Wasserturm. Wenn der Backstein im Licht steht, weiß sogar eine Regie, welche Tageszeit sie behauptet.",
        flag: "busker_park_hint",
        log: "Der Musiker empfiehlt das Parklicht am Wasserturm als Lichtanker.",
    },
    DialogueChoice {
        label: "Gibt es weitere Hinweise?",
        response: "Der Aushang an der Schanze erwähnt ein Kofferlicht. Mein Koffer hat zufällig eins. Dieses Zufällig ist verhandelbar.",
        flag: "busker_scope_hint",
        log: "Der Musiker deutet an, dass sein Kofferreflektor zur Außenreferenz gehört.",
    },
    DialogueChoice {
        label: "Kann ich den Reflektor leihen?",
        response: "Nicht ohne Nachweis. Wer aus einem Archivfall kommt, braucht entweder einen Entlassungsbogen oder eine sehr gute Ballade. Du wirkst formularstärker.",
        flag: "busker_reflector_requested",
        log: "Der Musiker verleiht den Kofferreflektor nur gegen sichtbaren Entlassungsnachweis.",
    },
];

const NURSE_CHOICES: &[DialogueChoice] = &[
    DialogueChoice {
        label: "Was ist passiert?",
        response: "Du hattest direkten Kontakt mit einem laufenden Rack. Die Werte sind stabil, aber du gehst erst nach Monitorcheck und Entlassungsbogen.",
        flag: "nurse_incident_explained",
        log: "Die Pflegekraft bestätigt den Stromschlag am Rack als realen Unfall.",
    },
    DialogueChoice {
        label: "Darf ich zurück?",
        response: "Wenn Monitor und Akte vollständig sind, bekommst du den Bogen. Danach gehst du nicht allein an offene Technik.",
        flag: "nurse_clearance_requested",
        log: "Die Station gibt eine bedingte Entlassung frei: Monitor prüfen, Bogen mitnehmen.",
    },
];

const SAFETY_CHOICES: &[DialogueChoice] = &[
    DialogueChoice {
        label: "Ist das Feuer gelöscht?",
        response: "Es gab keinen offenen Brand mehr, aber der Sprinklerkreis lief. Alarmfeld und Löschschrank müssen dokumentiert sein.",
        flag: "safety_clearance",
        log: "Der Sicherheitsdienst verlangt Alarm- und Materialprüfung vor Rückkehr ins Gebäude.",
    },
    DialogueChoice {
        label: "Kann ich zurück?",
        response: "Nach Alarmbestätigung und Schrankprüfung. Vorher bleibt die Tür zu.",
        flag: "safety_return_hint",
        log: "Rückkehr nur nach bestätigter Sicherheitsprüfung.",
    },
];

const DIALOGUES: &[Dialogue] = &[
    Dialogue {
        id: "mentor",
        speaker: "Aufnahmeleitung",
        opening: "Die Aufnahmeleitung bleibt am Studioeingang stehen und prüft eine Akte, deren Zeitstempel sich gerade selbst korrigiert.",
        choices: MENTOR_CHOICES,
    },
    Dialogue {
        id: "busker",
        speaker: "Straßenmusiker",
        opening: "Der Straßenmusiker steht am Durchgang, zeigt auf die Bahnhofsuhr und spielt etwas, das im Takt bleibt.",
        choices: BUSKER_CHOICES,
    },
    Dialogue {
        id: "nurse",
        speaker: "Pflegekraft",
        opening:
            "Die Pflegekraft bleibt sachlich und sieht erst auf den Monitor, dann in die Akte.",
        choices: NURSE_CHOICES,
    },
    Dialogue {
        id: "safety",
        speaker: "Sicherheitsdienst",
        opening:
            "Der Sicherheitsdienst steht unter dem Vordach und hält das nasse Protokoll trocken.",
        choices: SAFETY_CHOICES,
    },
];

fn window_conf() -> Conf {
    Conf {
        window_title: "Klixx".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        platform: Platform {
            webgl_version: WebGLVersion::WebGL2,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    install_game_font();
    let mut game = Game::new(load_state());
    let loaded_assets = load_game_assets().await;
    game.scene_textures = loaded_assets.scene_textures;
    game.player_texture = loaded_assets.player_texture;
    game.inventory_icons = loaded_assets.inventory_icons;
    game.verb_icons = loaded_assets.verb_icons;
    warm_font_cache();
    let mut first_frame = true;

    loop {
        let mouse = virtual_mouse();
        let clicked = is_mouse_button_pressed(MouseButton::Left);
        game.update(mouse, clicked);

        set_camera(&Camera2D {
            target: vec2(VW / 2.0, VH / 2.0),
            zoom: vec2(2.0 / VW, 2.0 / VH),
            ..Default::default()
        });
        game.draw(mouse);

        set_default_camera();
        next_frame().await;
        if first_frame {
            first_frame = false;
            hide_loading_overlay();
        }
    }
}

fn warm_font_cache() {
    let chars =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ÄÖÜäöüßẞ-.,:;!?()[]/% ";
    for size in [13, 14, 15, 16, 17, 18, 20, 22, 24, 30, 34] {
        let _ = measure_text(chars, None, size, 1.0);
    }
}

fn install_game_font() {
    if let Ok(font) = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/DejaVuSans.ttf")) {
        set_default_font(font);
    }
}

struct LoadedAssets {
    scene_textures: HashMap<&'static str, Texture2D>,
    player_texture: Option<Texture2D>,
    inventory_icons: Option<Texture2D>,
    verb_icons: Option<Texture2D>,
}

async fn load_game_assets() -> LoadedAssets {
    LoadedAssets {
        scene_textures: load_scene_textures().await,
        player_texture: load_pixel_texture("assets/sprites/player_walk.png").await,
        inventory_icons: load_pixel_texture("assets/sprites/inventory_icons.png").await,
        verb_icons: load_pixel_texture("assets/sprites/verb_icons.png").await,
    }
}

async fn load_scene_textures() -> HashMap<&'static str, Texture2D> {
    let mut textures = HashMap::new();
    for scene in SCENES {
        let path = format!("assets/scenes/{}.png", scene.id);
        match load_texture(&path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                textures.insert(scene.id, texture);
            }
            Err(error) => {
                eprintln!("Could not load scene asset {path}: {error}");
            }
        }
    }
    for (key, path) in SCENE_TEXTURE_VARIANTS {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                textures.insert(*key, texture);
            }
            Err(error) => {
                eprintln!("Could not load scene variant asset {path}: {error}");
            }
        }
    }
    textures
}

async fn load_pixel_texture(path: &str) -> Option<Texture2D> {
    match load_texture(path).await {
        Ok(texture) => {
            texture.set_filter(FilterMode::Nearest);
            Some(texture)
        }
        Err(error) => {
            eprintln!("Could not load sprite asset {path}: {error}");
            None
        }
    }
}

impl Game {
    fn new(state: GameState) -> Self {
        let modal = if state.complete {
            Modal::Milestone
        } else {
            Modal::None
        };
        let player_pos = scene_token_position(current_scene(&state.scene));
        Self {
            state,
            scene_textures: HashMap::new(),
            player_texture: None,
            inventory_icons: None,
            verb_icons: None,
            status: "Fall 000: Ein Host steckt in einem geöffneten Video-Frame. Stabilisiere Gegenwart, Stadtzeit und Clip-Anker.".to_string(),
            hover: None,
            modal,
            dialogue_response: None,
            player_pos,
            walk_target: None,
            player_facing: PlayerFacing::Down,
            death: None,
        }
    }

    fn update(&mut self, mouse: Vec2, clicked: bool) {
        if matches!(self.modal, Modal::None) && self.death.is_none() {
            self.step_player();
        }

        self.hover = None;
        if self.death.is_some() {
            if self
                .death
                .as_ref()
                .is_some_and(|death| runtime_time() - death.started_at >= 2.0)
            {
                self.complete_consequence_transition();
            }
            return;
        }

        if clicked {
            match self.modal {
                Modal::Dialogue(id) => {
                    self.update_dialogue(id, mouse);
                    return;
                }
                Modal::Milestone => {
                    if button_rect(540.0, 492.0, 200.0, 48.0).contains(mouse) {
                        self.modal = Modal::None;
                        self.status = "Der Rettungslauf ist freigegeben. Die Host-Spur bleibt stabil, solange niemand den Frame wieder anzüngelt.".to_string();
                    }
                    return;
                }
                Modal::None => {}
            }
        }

        for (index, verb) in Verb::ALL.iter().copied().enumerate() {
            let rect = verb_button_rect(index);
            if rect.contains(mouse) {
                self.hover = Some(verb.description().to_string());
                if clicked {
                    self.state.verb = verb;
                    self.state.selected_item = None;
                    self.status = format!("Verb gesetzt: {}.", verb.label());
                    save_state(&self.state);
                    return;
                }
            }
        }

        if button_rect(1162.0, 24.0, 92.0, 42.0).contains(mouse) {
            self.hover = Some("Fall neu starten".to_string());
            if clicked {
                self.reset();
                return;
            }
        }

        for (index, item_id) in self.state.inventory.clone().into_iter().enumerate() {
            if let Some(item) = item_meta(&item_id) {
                let rect = inventory_slot_rect(index);
                if rect.contains(mouse) {
                    self.hover = Some(format!("{} - {}", item.name, item.description));
                    if clicked {
                        if self.state.selected_item.as_deref() == Some(item.id) {
                            self.state.selected_item = None;
                            self.status = format!("{} abgelegt.", item.name);
                        } else {
                            self.state.selected_item = Some(item.id.to_string());
                            self.status = format!("{} ausgewählt.", item.name);
                        }
                        save_state(&self.state);
                        return;
                    }
                }
            }
        }

        let scene_rect = scene_rect();
        let scene = current_scene(&self.state.scene);
        let hotspots = self.hotspots();
        if let Some(hotspot) = hovered_hotspot(scene, &hotspots, scene_rect, mouse) {
            if let Some(item_id) = self.state.selected_item.as_deref() {
                let item = item_meta(item_id)
                    .map(|item| item.name)
                    .unwrap_or("Inventar");
                self.hover = Some(format!("{item} -> {}", hotspot.name));
            } else {
                self.hover = Some(format!(
                    "{}: {}",
                    self.state.verb.label(),
                    hotspot_action_name(self.state.verb, hotspot)
                ));
            }
            if clicked {
                self.walk_to_hotspot(scene, scene_rect, hotspot);
                self.handle_hotspot(hotspot.id);
                return;
            }
        }

        if clicked && scene_rect.contains(mouse) {
            self.set_walk_target(mouse);
            self.status = "Zielpunkt gesetzt.".to_string();
        }
    }

    fn set_walk_target(&mut self, target: Vec2) {
        let rect = scene_rect();
        let scene = current_scene(&self.state.scene);
        let clamped = vec2(
            target.x.clamp(rect.x + 36.0, rect.x + rect.w - 36.0),
            target.y.clamp(rect.y + 72.0, rect.y + rect.h - 8.0),
        );
        self.walk_target = Some(constrain_to_walkable(clamped, scene, rect));
    }

    fn walk_to_hotspot(&mut self, scene: &SceneMeta, scene_rect: Rect, hotspot: &HotspotSpec) {
        let rect = hotspot_bounds(scene, hotspot, scene_rect);
        self.set_walk_target(vec2(
            rect.x + rect.w * 0.5,
            (rect.y + rect.h + 16.0).min(scene_rect.y + scene_rect.h - 8.0),
        ));
    }

    fn step_player(&mut self) {
        let Some(target) = self.walk_target else {
            return;
        };
        let delta = target - self.player_pos;
        let distance = delta.length();
        let step = 300.0 * get_frame_time();
        if distance <= step || distance < 2.0 {
            self.player_pos = target;
            self.walk_target = None;
        } else {
            self.player_facing = facing_from_delta(delta);
            self.player_pos += delta / distance * step;
        }
    }

    fn reset_player_position(&mut self) {
        self.player_pos = scene_token_position(current_scene(&self.state.scene));
        self.walk_target = None;
    }

    fn update_dialogue(&mut self, id: &'static str, mouse: Vec2) {
        if dialogue_close_rect().contains(mouse) {
            self.modal = Modal::None;
            self.dialogue_response = None;
            return;
        }
        if let Some(dialogue) = dialogue(id) {
            let has_response = self
                .dialogue_response
                .as_ref()
                .is_some_and(|response| response.dialogue_id == id);
            for (i, choice) in dialogue.choices.iter().enumerate() {
                let rect = dialogue_choice_rect(i, has_response);
                if rect.contains(mouse) {
                    if id == "nurse" && choice.flag == "nurse_clearance_requested" {
                        self.state.flags.insert(choice.flag.to_string());
                        if self.flag("hospital_vitals_checked") {
                            self.state.flags.insert("nurse_clearance".to_string());
                            self.add_log(choice.log);
                            self.dialogue_response = Some(DialogueResponse {
                                dialogue_id: id,
                                text: "Die Werte sind stabil. Der Bogen ist freigegeben. Und die Zunge bleibt bitte im Mund.",
                            });
                        } else {
                            self.add_log(
                                "Die Pflegekraft verweigert die Freigabe bis zum Monitorcheck.",
                            );
                            self.dialogue_response = Some(DialogueResponse {
                                dialogue_id: id,
                                text: "Erst der Monitor. Ich unterschreibe nicht gegen ein Bauchgefühl, auch nicht gegen deins.",
                            });
                        }
                        self.status = format!("{} antwortet.", dialogue.speaker);
                        save_state(&self.state);
                        return;
                    }
                    if !choice.flag.is_empty() {
                        self.state.flags.insert(choice.flag.to_string());
                    }
                    if !choice.log.is_empty() {
                        self.add_log(choice.log);
                    }
                    self.dialogue_response = Some(DialogueResponse {
                        dialogue_id: id,
                        text: choice.response,
                    });
                    self.status = format!("{} antwortet.", dialogue.speaker);
                    save_state(&self.state);
                    return;
                }
            }
        }
    }

    fn handle_hotspot(&mut self, id: &str) {
        if self
            .hotspot(id)
            .is_some_and(|hotspot| hotspot.kind == HotspotKind::Exit || is_exit_like(id))
        {
            self.use_hotspot(id);
            save_state(&self.state);
            return;
        }

        if let Some(item) = self.state.selected_item.clone() {
            self.handle_item_use(id, &item);
            self.state.selected_item = None;
            save_state(&self.state);
            return;
        }

        self.perform_verb(id);
        save_state(&self.state);
    }

    fn perform_verb(&mut self, id: &str) {
        match self.state.verb {
            Verb::Look => self.look(id),
            Verb::Poke => self.poke(id),
            Verb::Use => self.use_hotspot(id),
            Verb::PickUp => self.pick_up_hotspot(id),
            Verb::Tongue => self.tongue(id),
            Verb::Smell => self.smell(id),
            Verb::Talk => self.talk(id),
        }
    }

    #[allow(dead_code)]
    fn smart_interact(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            self.status = "Dieser Punkt ist in dieser Szene nicht aktiv.".to_string();
            return;
        };

        match hotspot.kind {
            HotspotKind::Exit => self.use_hotspot(id),
            HotspotKind::Pickup => self.pick_up_hotspot(id),
            HotspotKind::Character => {
                if hotspot.talk_id.is_some() {
                    self.talk(id);
                } else {
                    self.inspect(id);
                }
            }
            HotspotKind::Prop => match id {
                "greenscreen_setup" => self.use_hotspot(id),
                _ => self.inspect(id),
            },
        }
    }

    #[allow(dead_code)]
    fn look(&mut self, id: &str) {
        if let Some(hotspot) = self.hotspot(id) {
            self.observe_hotspot(id, hotspot.look);
        }
    }

    fn poke(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            return;
        };

        match id {
            "server_racks" | "video_hub" | "sdi_spool" | "graphics_terminal" | "on_air_lamp" => {
                self.trigger_consequence(
                    DeathKind::Shock,
                    "Du stubst in laufende Technik. Das Signal findet einen schnelleren Weg durch dich.",
                );
            }
            "call_button" => self.call_nurse("Du stubst den Rufknopf mit einem Finger an. Das System akzeptiert die primitive Schnittstelle."),
            "hospital_monitor" => {
                self.status = "Der Monitor piept einmal beleidigt. Medizinische Geräte mögen keine Meinungsumfragen per Finger.".to_string();
            }
            "nurse_station" => {
                self.status = "Die Pflegekraft tritt einen halben Schritt zurück. Der Befund lautet: Sozialer Abstand stabil.".to_string();
            }
            "discharge_clipboard" => {
                self.status = "Das Klemmbrett klackt. Es wirkt dadurch amtlicher, aber nicht freigegebener.".to_string();
            }
            "hospital_exit" => {
                self.status = "Die Korridortür bewegt sich keinen Millimeter. Sie ist eine Tür, kein Gong.".to_string();
            }
            "busker_case" => {
                self.status = "Der Koffer scheppert. Der Straßenmusiker nickt im Takt, aber nicht im Sinne einer Zustimmung.".to_string();
            }
            "corner_kiosk" => {
                self.status = "Der Kiosk antwortet mit dem dumpfen Geräusch geschlossener Ladenöffnungszeiten.".to_string();
            }
            "record_store" => {
                self.status = "Die Scheibe vibriert kurz. Drinnen fällt kein einziges Urteil über deinen Musikgeschmack um.".to_string();
            }
            "mold_rack" | "casting_table" | "set_pieces" => self.trigger_consequence(
                DeathKind::Fall,
                "Du bringst eine instabile Reihe aus dem Gleichgewicht. Die Szene endet unter Material.",
            ),
            _ => {
                self.status = poke_response(id, hotspot.name).to_string();
            }
        }
    }

    fn tongue(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            return;
        };

        match id {
            "server_racks" | "video_hub" | "graphics_terminal" | "on_air_lamp" => {
                self.trigger_consequence(
                    DeathKind::Shock,
                    "Du berührst laufende Technik mit der Zunge. Der Stromschlag ist real.",
                );
            }
            "rehearsal_monitor" => {
                self.trigger_consequence(
                    DeathKind::Signal,
                    "Du anzüngelst den Anker-Monitor. Der offene Frame kippt in Störsignal, und die Rettungsroutine zieht dich ins Archiv.",
                );
            }
            "nurse_station" => {
                self.status = "Die Pflegekraft sagt ohne Pause: \"Nein.\" Danach schreibt sie etwas in die Akte, das du nicht lesen möchtest.".to_string();
            }
            "call_button" => {
                self.call_nurse("Du drückst den Rufknopf mit der Zunge. Der Knopf funktioniert. Deine Würde nicht.");
            }
            "hospital_monitor" => {
                self.status = "Die Pflegekraft ruft aus dem Flur: \"Der Monitor misst Werte. Er sammelt keine Speichelproben.\"".to_string();
            }
            "discharge_clipboard" => {
                self.status = "Der Entlassungsbogen schmeckt nach Papier, Toner und einer schlechten Entscheidung mit Durchschlag.".to_string();
            }
            "hospital_exit" => {
                self.status = "Die Türklinke schmeckt nach Desinfektionsmittel und Verwaltungsgebäude. Beides bleibt im Gedächtnis.".to_string();
            }
            "busker_case" => {
                self.status = "Der Straßenmusiker zieht den Koffer weg. \"Münzen ja. Zunge nein.\""
                    .to_string();
            }
            "market_boxes" => {
                self.status = "Die Pappe schmeckt nach Regen, Tomatenkiste und einem Umzug, der nie stattgefunden hat.".to_string();
            }
            "corner_kiosk" => {
                self.status = "Das Kioskfenster schmeckt nach Stadtstaub. Das Band bleibt trotzdem hinter der Scheibe.".to_string();
            }
            "greenscreen_wall" | "floor_marks" | "gaffer_roll" | "chat_preview" => {
                self.trigger_consequence(
                    DeathKind::Fire,
                    "Du probierst Produktionsmaterial mit der Zunge, reißt reflexartig am Aufbau und löst den Sprinkleralarm aus.",
                );
            }
            "mold_rack" | "mold_token" => {
                self.set_flag_log(
                    "mold_material_checked",
                    "Die Materialprobe ist über Oberfläche und Geschmack zugeordnet.",
                    "Die Frame-Probe ist zugeordnet. Das Protokoll erwähnt nicht die Methode.",
                );
            }
            _ => {
                self.status = tongue_response(id, hotspot.name).to_string();
            }
        }
    }

    fn smell(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            return;
        };

        match id {
            "nurse_station" => {
                self.status = "Die Pflegekraft riecht nach Desinfektionsmittel, Kaffee und zwölf Stunden professioneller Geduld.".to_string();
            }
            "call_button" => {
                self.status = "Der Rufknopf riecht nach Plastik und Menschen, die zu spät beschlossen haben, Hilfe zu brauchen.".to_string();
            }
            "hospital_monitor" => {
                self.status = "Der Monitor riecht nach warmem Staub. Das ist kein Vitalwert, aber immerhin reproduzierbar.".to_string();
            }
            "discharge_clipboard" => {
                self.status = "Der Bogen riecht nach Formularschrank. Du bekommst kurz den Drang, in Blockschrift zu gehen.".to_string();
            }
            "hospital_exit" => {
                self.status =
                    "Aus dem Korridor kommt Suppe, Reinigungsmittel und eine sehr müde Durchsage."
                        .to_string();
            }
            "busker_case" => {
                self.status = "Der Koffer riecht nach Münzen, altem Filz und einem Reflektor, der viel zu wichtig geworden ist.".to_string();
            }
            "record_store" => {
                self.status = "Der Plattenladen riecht nach Papierhüllen und Menschen, die Pressungen erklären.".to_string();
            }
            "mold_rack" | "mold_token" => self.set_flag_log(
                "mold_material_checked",
                "Der Geruch bestätigt: Die helle Probe gehört zur frischen Formenreihe.",
                "Die Frame-Probe ist über Materialgeruch zugeordnet.",
            ),
            "factory_floor" => {
                self.status =
                    "Der Boden riecht nach feuchtem Material, nicht nach einem Ausgang.".to_string()
            }
            "greenscreen_wall" => {
                self.status = "Der Stoff riecht nach trockenem Molton und Staub.".to_string();
            }
            _ => {
                self.status = smell_response(id, hotspot.name).to_string();
            }
        }
    }

    fn pick_up_hotspot(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            self.status = "Dieser Punkt ist in dieser Szene nicht aktiv.".to_string();
            return;
        };

        match id {
            "dispo_board" => self.take_item(
                "call_sheet",
                "call_sheet_taken",
                "Fallakte 000 aufgehoben.",
                "Fallakte 000 gesichert: Studio, Regie, Stadtzeit und Schimmelbrüder-Frame sind als Anker vermerkt.",
            ),
            "gaffer_roll" => self.take_item(
                "gaffer_tape",
                "gaffer_tape_taken",
                "Gaffer-Tape aufgehoben.",
                "Gaffer-Tape aus Haus 15 gesichert.",
            ),
            "sdi_label_printer" => self.take_item(
                "sdi_label",
                "sdi_label_taken",
                "SDI-Label aufgehoben.",
                "Das SDI-Label benennt die Route von Serverraum 11 zum Greenscreen-Studio.",
            ),
            "print_shop" => self.take_item(
                "lower_third_card",
                "lower_third_card_taken",
                "Host-Karte aufgehoben.",
                "Die freigegebene Host-Karte wurde im Copyshop abgeholt.",
            ),
            "city_reflector" => {
                self.status = "Der Parkreflektor ist festgeschraubt. Der mobile Reflektor hängt am Koffer des Straßenmusikers.".to_string();
            }
            "market_boxes" => self.take_item(
                "cardboard_piece",
                "cardboard_piece_taken",
                "Pappstück aufgehoben.",
                "Ein stabiles Pappstück aus den Marktkisten wurde als Schildrohling gesichert.",
            ),
            "corner_kiosk" => self.take_item(
                "clear_tape",
                "clear_tape_taken",
                "Transparentband aufgehoben.",
                "Das kurze Transparentband vom Kiosk ist jetzt Teil der Außenlogistik.",
            ),
            "mold_token" => self.take_item(
                "mold_token",
                "mold_token_taken",
                "Frame-Probe aufgehoben.",
                "Die Frame-Probe aus der Schimmelbrüder-Halle wurde gesichert.",
            ),
            "discharge_clipboard" => self.try_take_medical_release(),
            "checksum_printout" => self.take_item(
                "checksum_note",
                "checksum_note_taken",
                "Prüfsummenzettel aufgehoben.",
                "Der Prüfsummenzettel aus dem Archiv-Rettungsplatz wurde gesichert.",
            ),
            _ if hotspot.kind == HotspotKind::Pickup => {
                self.status = format!("{} lässt sich noch nicht sinnvoll aufheben.", hotspot.name);
            }
            _ => {
                self.status = pickup_response(id, hotspot.name).to_string();
            }
        }
    }

    fn call_nurse(&mut self, status: &str) {
        if self.flag("nurse_called") {
            self.status = "Der Ruf ist schon raus. Die Pflegekraft steht im Zimmer und ist jetzt vor allem über die Wiederholung informiert.".to_string();
            return;
        }
        self.set_flag_log(
            "nurse_called",
            "Der Rufknopf wurde ausgelöst. Die Pflegekraft kommt ins Zimmer.",
            status,
        );
    }

    fn use_hotspot(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            self.status = "Dieser Punkt ist in dieser Szene nicht aktiv.".to_string();
            return;
        };

        if hotspot.kind == HotspotKind::Exit {
            if id == "video_schimmelbrueder" {
                self.enter_schimmel_video();
                return;
            }
            self.travel(
                hotspot.id,
                &format!("Du gehst zu {}.", current_scene(hotspot.id).name),
            );
            return;
        }

        match id {
            "call_button" => {
                self.call_nurse("Der Rufknopf wurde benutzt. Die Pflegekraft kommt ins Zimmer.")
            }
            "hospital_exit" => self.try_leave_hospital(),
            "alarm_panel" => self.set_flag_log(
                "fire_alarm_logged",
                "Der Sprinklerkreis ist im Alarmfeld bestätigt.",
                "Alarmkreis bestätigt.",
            ),
            "extinguisher_cabinet" => self.set_flag_log(
                "fire_cabinet_checked",
                "Der Löschschrank ist vollständig. Es gab keine offene Brandstelle mehr.",
                "Löschschrank geprüft.",
            ),
            "fire_return_door" => self.try_return_after_fire(),
            "brace_beam" => self.set_flag_log(
                "collapse_braced",
                "Die Stützstrebe sitzt fest genug für einen kurzen Durchgang.",
                "Strebe gesichert.",
            ),
            "release_rope" => self.set_flag_log(
                "collapse_rope_released",
                "Die Notleine bewegt den Riegel über dem Spalt.",
                "Riegel gelöst.",
            ),
            "workshop_gap" | "collapse_exit" => self.try_escape_collapse(),
            "recovery_terminal" => self.try_close_recovery_session(),
            "control_room_return" => self.try_return_from_recovery(),
            "song_exit_gate" => self.try_open_schimmel_exit(),
            "greenscreen_setup" => self.try_finish_loop(),
            _ if hotspot.talk_id.is_some() => self.talk(id),
            _ if hotspot.kind == HotspotKind::Pickup => {
                let target = pickup_target_name(id).unwrap_or(hotspot.name);
                self.status = format!("{target} ist ein Gegenstand. Nimm ihn mit Aufheben.");
            }
            _ => {
                self.status = use_response(id, hotspot.name).to_string();
            }
        }
    }

    fn inspect(&mut self, id: &str) {
        if let Some(hotspot) = self.hotspot(id) {
            self.observe_hotspot(id, hotspot.inspect);
        }
    }

    fn observe_hotspot(&mut self, id: &str, fallback: &str) {
        match id {
            "dispo_board" => self.set_flag_log(
                "dispo_checked",
                "Die Falltafel bestätigt: Studio, Regie, Stadtzeit und Schimmelbrüder-Frame.",
                "Fall 000 bestätigt die Reihenfolge der Anker.",
            ),
            "route_map" => self.set_flag_log(
                "campus_mapped",
                "Der Gebäudeplan sortiert Haus 9, Haus 11 und Haus 15.",
                "Heinrichstraße verstanden: Studio, Serverraum, Szenenbau und ein Ausgang in die Stadt.",
            ),
            "video_hub" => self.set_flag_log(
                "route_checked",
                "Die Kreuzschiene bestätigt die Route von Haus 11 ins Studio.",
                "Der Video-Hub bestätigt die aktive Signalroute. Das Label fehlt noch.",
            ),
            "station_clock" => self.set_flag_log(
                "station_timed",
                "Die Bahnhofsuhr wurde als Zeitreferenz dokumentiert.",
                "Zeitreferenz an der Sternschanze dokumentiert.",
            ),
            "water_tower" => self.set_flag_log(
                "city_light_checked",
                "Das Licht am Wasserturm wurde als Referenz dokumentiert.",
                "Lichtreferenz im Schanzenpark dokumentiert.",
            ),
            "city_notice" => self.set_flag_log(
                "city_notice_checked",
                "Der Drehhinweis nennt Bahnhofsuhr, Wasserturmlicht und ein reflektierendes Kofferschild am Durchgang.",
                "Der Aushang verweist auf den Kofferreflektor des Straßenmusikers.",
            ),
            "graphics_terminal" => self.set_flag_log(
                "graphics_slot_checked",
                "Der Grafikplatz wartet auf die freigegebene Host-Karte.",
                "Die Regie benötigt die freigegebene Host-Karte.",
            ),
            "factory_floor" => self.set_flag_log(
                "schimmel_floor_checked",
                "Der Hallenboden ist breit genug, damit die Figur zwischen den Formen stehen kann.",
                "Standfläche im Schimmelbrüder-Frame bestätigt.",
            ),
            "mold_rack" => self.set_flag_log(
                "mold_row_checked",
                "Die Formenreihe folgt der Abfolge leer, voll, leer, voll.",
                "Formenfolge im Schimmelbrüder-Frame dokumentiert.",
            ),
            "patterned_belt" => self.set_flag_log(
                "mold_pattern_checked",
                "Die Musterbahn wiederholt die Abfolge der Formen.",
                "Musterbahn als Zuordnung zur Formenfolge dokumentiert.",
            ),
            "hospital_monitor" => self.set_flag_log(
                "hospital_vitals_checked",
                "Der Monitor zeigt stabile Werte nach dem Stromschlag.",
                "Die Werte sind stabil.",
            ),
            "alarm_panel" => self.set_flag_log(
                "fire_alarm_seen",
                "Das Alarmfeld zeigt den ausgelösten Sprinklerkreis.",
                "Der Alarmkreis ist identifiziert. Benutze das Feld zur Bestätigung.",
            ),
            "brace_beam" => self.set_flag_log(
                "collapse_brace_seen",
                "Die Strebe ist der einzige belastbare Punkt im gefallenen Material.",
                "Die Strebe muss gesichert werden.",
            ),
            "archive_drive" => self.set_flag_log(
                "archive_drive_checked",
                "Das Archivlaufwerk ist lesbar. Die Prüfsumme muss noch zur Session.",
                "Das Laufwerk ist lesbar.",
            ),
            "recovery_terminal" => self.set_flag_log(
                "recovery_terminal_seen",
                "Das Rettungsterminal wartet auf die Prüfsumme.",
                "Das Terminal braucht den Prüfsummenzettel.",
            ),
            _ => {
                self.status = fallback.to_string();
            }
        }
    }

    fn talk(&mut self, id: &str) {
        if id == "nurse_station" && !self.flag("nurse_called") {
            self.status =
                "Benutze zuerst den Rufknopf. Die Pflegekraft kommt nicht ohne Signal.".to_string();
            return;
        }
        if let Some(dialogue_id) = self.hotspot(id).and_then(|hotspot| hotspot.talk_id) {
            self.modal = Modal::Dialogue(dialogue_id);
            self.dialogue_response = None;
        } else {
            let name = self
                .hotspot(id)
                .map(|hotspot| hotspot.name)
                .unwrap_or("Hier");
            self.status = talk_response(id, name).to_string();
        }
    }

    fn handle_item_use(&mut self, hotspot_id: &str, item_id: &str) {
        match (hotspot_id, item_id) {
            ("greenscreen_wall" | "floor_marks" | "greenscreen_setup", "gaffer_tape") => {
                self.set_flag_log(
                    "greenscreen_marked",
                    "Neue Bodenmarken wurden im Studio gesetzt.",
                    "Bodenmarken gesetzt.",
                );
            }
            ("video_hub" | "sdi_spool" | "greenscreen_setup", "sdi_label") => {
                self.set_flag_log(
                    "route_labeled",
                    "Das SDI-Label klebt an der Route von Haus 11 Richtung Studio.",
                    "Signalweg beschriftet.",
                );
            }
            ("graphics_terminal" | "chat_preview" | "greenscreen_setup", "lower_third_card") => {
                self.set_flag_log(
                    "graphic_loaded",
                    "Die Host-Karte ist am Grafikplatz eingepflegt.",
                    "Host-Karte geladen.",
                );
            }
            ("dispo_board" | "mentor_shadow" | "intercom_voice", "call_sheet") => {
                self.set_flag_log(
                    "case_briefed",
                    "Die Fallakte 000 wurde mit der Produktionsleitung abgeglichen.",
                    "Fallakte abgeglichen: Die Ankerliste ist verbindlich.",
                );
            }
            ("route_map", "call_sheet") => {
                self.set_flag_log(
                    "campus_mapped",
                    "Die Fallakte wurde mit dem Gebäudeplan abgeglichen: Haus 9, Haus 11 und Haus 15 bilden die Studio-Gegenwart.",
                    "Fallroute auf dem Gebäudeplan markiert.",
                );
            }
            ("station_clock", "call_sheet") => {
                self.set_flag_log(
                    "station_timed",
                    "Die Bahnhofsuhr wurde in Fallakte 000 als harte Zeitmarke eingetragen.",
                    "Zeitmarke in der Fallakte notiert.",
                );
            }
            ("water_tower", "call_sheet") => {
                self.set_flag_log(
                    "city_light_checked",
                    "Das Wasserturmlicht wurde in Fallakte 000 als Lichtreferenz eingetragen.",
                    "Lichtreferenz in der Fallakte notiert.",
                );
            }
            ("camera_one" | "greenscreen_setup", "city_reflector") => {
                self.set_flag_log(
                    "city_reflector_placed",
                    "Der Lichtreflektor wurde am Studio-Setup platziert.",
                    "Stadtlicht am Rückholpunkt platziert.",
                );
            }
            ("mold_rack", "mold_token") => {
                self.set_flag_log(
                    "mold_material_checked",
                    "Die Frame-Probe wurde mit der Formenreihe abgeglichen.",
                    "Frame-Probe passt zur Formenreihe.",
                );
            }
            ("patterned_belt", "mold_token") => {
                self.set_flag_log(
                    "mold_pattern_checked",
                    "Die Frame-Probe wurde an der Musterbahn ausgerichtet.",
                    "Musterfolge auf die Frame-Probe übertragen.",
                );
            }
            ("print_shop", "cardboard_piece") => {
                self.try_print_busker_sign();
            }
            ("busker_case", "medical_release") => {
                self.try_show_release_to_busker();
            }
            ("busker_case", "busker_sign") => {
                self.try_place_busker_sign();
            }
            ("busker_case", "clear_tape") => {
                self.try_tape_busker_sign();
            }
            ("greenscreen_setup", "call_sheet") => {
                self.set_flag_log(
                    "call_sheet_checked",
                    "Die Fallakte 000 liegt am Setup.",
                    "Fallakte am Setup geprüft.",
                );
                self.try_finish_loop();
            }
            ("song_exit_gate", "mold_token") => {
                self.try_open_schimmel_exit();
            }
            ("hospital_exit", "medical_release") => {
                self.try_leave_hospital();
            }
            ("recovery_terminal", "checksum_note") => {
                self.try_close_recovery_session();
            }
            ("archive_drive", "checksum_note") => {
                self.set_flag_log(
                    "archive_drive_checked",
                    "Die Prüfsumme passt zum Archivlaufwerk. Die Rettungssession hat eine Quelle.",
                    "Prüfsumme am Laufwerk bestätigt.",
                );
            }
            ("rehearsal_monitor" | "video_schimmelbrueder", "checksum_note") => {
                self.status = "Die Prüfsumme gehört zur Rettungsstation, nicht zum geöffneten Frame. Der Monitor reagiert mit sehr altem Rauschen.".to_string();
            }
            ("nurse_station", "medical_release") => {
                self.status = "Die Pflegekraft prüft den Entlassungsbogen und sagt: \"Gut. Dann bitte heute keine zweite Fachabteilung eröffnen.\"".to_string();
            }
            ("safety_officer", "medical_release") => {
                self.status = "Der Sicherheitsdienst akzeptiert den Entlassungsbogen nicht als Brandschutzprotokoll, findet ihn aber dramaturgisch bemerkenswert.".to_string();
            }
            _ => {
                let item = item_meta(item_id).map(|i| i.name).unwrap_or("Das");
                self.status = funny_item_mismatch(hotspot_id, item_id)
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        format!(
                            "{item} passt hier nicht. Es wirkt kurz wie Adventure-Logik, ist aber nur Inventar-Akrobatik."
                        )
                    });
            }
        }
    }

    fn try_print_busker_sign(&mut self) {
        if self.has_item("busker_sign") {
            self.status =
                "Der Copyshop hat das Kofferschild schon gedruckt. Mehr Laminat wäre Angeberei."
                    .to_string();
            return;
        }
        if !self.flag("city_notice_checked") {
            self.status =
                "Der Copyshop braucht den Wortlaut vom Drehhinweis an der Schanze.".to_string();
            return;
        }
        self.take_item(
            "busker_sign",
            "busker_sign_printed",
            "Kofferschild gedruckt.",
            "Der Copyshop druckt ein wetterfestes Schild für den Straßenmusiker-Koffer.",
        );
    }

    fn try_show_release_to_busker(&mut self) {
        self.set_flag_log(
            "busker_release_shown",
            "Der Straßenmusiker akzeptiert den Entlassungsbogen als Nachweis, dass der Stromschlag offiziell überlebt wurde.",
            "Der Musiker liest den Entlassungsbogen und nickt: offiziell lebendig reicht als Bonität.",
        );
        self.state
            .flags
            .insert("busker_reflector_requested".to_string());
    }

    fn try_place_busker_sign(&mut self) {
        if !self.flag("busker_release_shown") {
            self.status = "Der Musiker will erst den Entlassungsbogen sehen. Fremde Schilder bekommt er täglich.".to_string();
            return;
        }
        self.set_flag_log(
            "busker_sign_positioned",
            "Das neue Schild liegt am Koffer des Straßenmusikers. Es braucht nur noch Klebeband.",
            "Das Schild liegt richtig. Ohne Klebeband ist es nur ein sehr kurzes Flugblatt.",
        );
    }

    fn try_tape_busker_sign(&mut self) {
        if !self.flag("busker_release_shown") {
            self.status = "Der Musiker lässt dich nicht an den Koffer, solange er deinen Entlassungsbogen nicht gesehen hat.".to_string();
            return;
        }
        if !self.flag("busker_sign_positioned") {
            self.status =
                "Transparentband direkt auf Filz ist keine Reparatur, sondern moderne Kunst."
                    .to_string();
            return;
        }
        self.finish_busker_reflector();
    }

    fn finish_busker_reflector(&mut self) {
        if self.flag("busker_case_fixed") || self.has_item("city_reflector") {
            self.status =
                "Der Koffer ist beschildert. Der Reflektor ist bereits geliehen.".to_string();
            return;
        }
        self.state.flags.insert("busker_case_fixed".to_string());
        self.add_item("city_reflector");
        self.add_log("Der Straßenmusiker-Koffer ist neu beschildert. Der Kofferreflektor geht als Lichtreferenz mit.");
        self.status =
            "Der Musiker löst den Reflektor vom Koffer: \"Bring ihn zurück, bevor er berühmt wird.\""
                .to_string();
    }

    fn try_take_medical_release(&mut self) {
        if !self.flag("hospital_vitals_checked") {
            self.status = "Prüfe zuerst den Monitor. Ohne stabile Werte gibt es keine Entlassung."
                .to_string();
            return;
        }
        if !self.flag("nurse_clearance_requested") {
            self.status =
                "Sprich mit der Pflegekraft. Formulare entstehen hier nicht durch Blickkontakt."
                    .to_string();
            return;
        }
        if !self.flag("nurse_clearance") {
            self.status =
                "Sprich noch einmal mit der Pflegekraft. Jetzt liegen die Monitorwerte vor."
                    .to_string();
            return;
        }
        self.take_item(
            "medical_release",
            "medical_release_taken",
            "Entlassungsbogen aufgehoben.",
            "Der Entlassungsbogen bestätigt: Stromschlag überstanden, Rückkehr nur mit Abstand zu offenen Racks.",
        );
    }

    fn try_leave_hospital(&mut self) {
        if !self.has_item("medical_release") {
            self.status =
                "Die Stationsschleuse bleibt ohne Entlassungsbogen geschlossen.".to_string();
            return;
        }
        self.travel(
            "office_hall",
            "Du kommst mit Entlassungsbogen zurück ins Gebäude. Offene Technik bleibt tabu.",
        );
    }

    fn try_return_after_fire(&mut self) {
        if !self.flag("fire_alarm_logged") {
            self.status =
                "Bestätige zuerst den ausgelösten Sprinklerkreis am Alarmfeld.".to_string();
            return;
        }
        if !self.flag("fire_cabinet_checked") {
            self.status =
                "Prüfe den Löschschrank. Der Sicherheitsdienst braucht den Bestand.".to_string();
            return;
        }
        if !self.flag("safety_clearance") {
            self.status = "Der Sicherheitsdienst muss die Rückkehr freigeben.".to_string();
            return;
        }
        self.travel(
            "building_courtyard",
            "Der Sicherheitsdienst gibt den Rückweg frei. Das Protokoll bleibt im Hof.",
        );
    }

    fn try_escape_collapse(&mut self) {
        if !self.flag("collapse_braced") {
            self.status =
                "Die gefallenen Teile müssen erst über die Strebe gesichert werden.".to_string();
            return;
        }
        if !self.flag("collapse_rope_released") {
            self.status =
                "Der Riegel über dem Spalt hängt noch fest. Benutze die Notleine.".to_string();
            return;
        }
        self.travel(
            "set_workshop",
            "Du kommst aus dem blockierten Bereich zurück in den Szenenbau.",
        );
    }

    fn try_close_recovery_session(&mut self) {
        if !self.has_item("checksum_note") {
            self.status = "Das Terminal verlangt die Prüfsumme vom Ausdruck.".to_string();
            return;
        }
        if !self.flag("archive_drive_checked") {
            self.status =
                "Prüfe zuerst das Archivlaufwerk. Die Session braucht eine Quelle.".to_string();
            return;
        }
        self.set_flag_log(
            "recovery_session_closed",
            "Die Prüfsumme wurde am Rettungsterminal bestätigt.",
            "Die Rettungssession ist abgeschlossen.",
        );
    }

    fn try_return_from_recovery(&mut self) {
        if !self.flag("recovery_session_closed") {
            self.status = "Schließe zuerst die Rettungssession am Terminal ab.".to_string();
            return;
        }
        self.travel(
            "control_room",
            "Du kehrst mit sauber abgeschlossener Rettungssession in die Regie zurück.",
        );
    }

    fn enter_schimmel_video(&mut self) {
        self.state.flags.insert("video_entry_started".to_string());
        self.add_log("Die Regie rastet den Schimmelbrüder-Frame ein. Der Monitor wird als begehbarer Raum und als fraglicher Zeitpunkt geöffnet.");
        self.travel(
            "video_schimmelbrueder",
            "Das Monitorbild hält an, der Ton fällt weg, und die Figur tritt in den Schimmelbrüder-Frame.",
        );
    }

    fn try_open_schimmel_exit(&mut self) {
        if self.flag("schimmel_video_cleared") {
            self.travel(
                "control_room",
                "Der Clip-Ausgang klappt zurück in die Regie. Im Monitor bleibt eine stabile Host-Spur stehen.",
            );
            return;
        }
        if !self.flag("schimmel_floor_checked") {
            self.status =
                "Prüfe zuerst die Standfläche der Figur in dieser Fertigungshalle.".to_string();
            return;
        }
        if !self.has_item("mold_token") {
            self.status =
                "Dem Clip-Ausgang fehlt eine Frame-Probe aus der Formenreihe.".to_string();
            return;
        }
        if !self.flag("mold_material_checked") {
            self.status =
                "Die Probe ist noch nicht zugeordnet. Gleiche sie mit der Formenreihe ab, bevor du sie am Ausgang benutzt."
                    .to_string();
            return;
        }
        if !self.flag("mold_pattern_checked") {
            self.status =
                "Die Probe allein reicht nicht. Die Musterbahn definiert die Reihenfolge."
                    .to_string();
            return;
        }
        self.state
            .flags
            .insert("schimmel_video_cleared".to_string());
        self.add_log(
            "Die Frame-Probe wurde dem Clip-Ausgang zugeordnet. Der Schimmelbrüder-Frame hält jetzt eine stabile Host-Spur.",
        );
        self.status =
            "Frame-Probe zugeordnet. Der Host-Echo ist stabil, der Clip-Ausgang ist freigegeben."
                .to_string();
    }

    fn try_finish_loop(&mut self) {
        if !self.has_item("call_sheet") {
            self.status = "Die Fallakte fehlt. Hol sie an der Falltafel im Büroflur.".to_string();
            return;
        }
        if !self.flag("greenscreen_marked") {
            self.status =
                "Die Bodenmarken fehlen noch. Nutze das Gaffer-Tape am Rückholpunkt.".to_string();
            return;
        }
        if !self.flag("route_labeled") {
            self.status =
                "Der Signalweg ist noch nicht beschriftet. Das SDI-Label gehört an die Route."
                    .to_string();
            return;
        }
        if !self.flag("graphic_loaded") {
            self.status = "Die Host-Karte fehlt am Grafikplatz.".to_string();
            return;
        }
        if !self.flag("schimmel_video_cleared") {
            self.status =
                "Der Schimmelbrüder-Frame ist noch instabil. Öffne ihn in der Regie und bringe die Host-Spur zurück."
                    .to_string();
            return;
        }
        if !self.flag("station_timed") {
            self.status = "Die Zeitreferenz fehlt. Prüfe die Uhr an der Sternschanze.".to_string();
            return;
        }
        if !self.flag("city_light_checked") {
            self.status =
                "Das Parklicht fehlt. Schau dir den Wasserturm im Schanzenpark an.".to_string();
            return;
        }
        if !self.flag("city_reflector_placed") {
            self.status =
                "Die Lichtreferenz ist noch nicht gesetzt. Nutze den Reflektor im Studio."
                    .to_string();
            return;
        }
        self.state.complete = true;
        self.state.flags.insert("rescue_ready".to_string());
        self.add_log("Der Rettungslauf ist vollständig vorbereitet: Gegenwart, Stadtzeit, Host-Karte und Clip-Anker sind abgeglichen.");
        self.status = "Die Regie bestätigt den Rettungslauf.".to_string();
        self.modal = Modal::Milestone;
    }

    fn draw(&self, mouse: Vec2) {
        if self.death.is_some() {
            self.draw_black_transition();
            return;
        }

        draw_background();
        self.draw_topbar(mouse);
        self.draw_scene(mouse);
        self.draw_footer(mouse);
        if let Some(text) = &self.hover {
            draw_hover(text, mouse);
        }
        match self.modal {
            Modal::None => {}
            Modal::Dialogue(id) => self.draw_dialogue(id, mouse),
            Modal::Milestone => self.draw_milestone(mouse),
        }
    }

    fn draw_black_transition(&self) {
        clear_background(BLACK);
        draw_rectangle(0.0, 0.0, VW, VH, BLACK);
    }

    fn draw_topbar(&self, mouse: Vec2) {
        let topbar = Rect::new(20.0, 18.0, 1240.0, 58.0);
        draw_rectangle(topbar.x, topbar.y, topbar.w, topbar.h, ui_dark());
        draw_rectangle_lines(topbar.x, topbar.y, topbar.w, topbar.h, 2.0, rust());

        let scene = current_scene(&self.state.scene);
        draw_rectangle(28.0, 24.0, 322.0, 46.0, Color::new(0.03, 0.025, 0.02, 0.58));
        draw_rectangle(
            370.0,
            25.0,
            735.0,
            44.0,
            Color::new(0.03, 0.025, 0.02, 0.55),
        );
        draw_text_ex(scene.zone, 34.0, 41.0, text_params(18, ochre()));
        draw_text_ex(scene.name, 34.0, 66.0, text_params(24, paper()));
        draw_text_wrapped(&self.status, 380.0, 42.0, 700.0, 19.0, bone());
        button(button_rect(1162.0, 24.0, 92.0, 42.0), "Neu", false, mouse);
    }

    fn draw_scene(&self, mouse: Vec2) {
        let rect = scene_rect();
        let scene = current_scene(&self.state.scene);
        self.draw_scene_background(scene, rect);

        let hotspots = self.hotspots();
        if let Some(hotspot) = hovered_hotspot(scene, &hotspots, rect, mouse) {
            draw_hotspot_hover(scene, hotspot, rect);
        }

        if let Some(target) = self.walk_target {
            draw_placeholder_sprite(
                Rect::new(target.x - 20.0, target.y - 10.0, 40.0, 14.0),
                "GEHEN",
                PlaceholderCategory::Ui,
                true,
            );
        }

        self.draw_player(scene, rect);
    }

    fn draw_player(&self, scene: &SceneMeta, scene_rect: Rect) {
        let token = self.player_pos;
        let scale = player_depth_scale(token, scene, scene_rect);
        let frame = if self.walk_target.is_some() {
            walk_cycle_frame(get_time())
        } else {
            0.0
        };

        if let Some(texture) = &self.player_texture {
            let frame_w = texture.width() / 4.0;
            let frame_h = texture.height() / 4.0;
            let dest = vec2(PLAYER_DRAW_W * scale, PLAYER_DRAW_H * scale);
            draw_texture_ex(
                texture,
                token.x - dest.x * 0.5,
                token.y - dest.y * PLAYER_FOOT_ANCHOR_Y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(
                        frame * frame_w,
                        self.player_facing.row() * frame_h,
                        frame_w,
                        frame_h,
                    )),
                    dest_size: Some(dest),
                    ..Default::default()
                },
            );
            return;
        }

        let fallback = vec2(48.0 * scale, 96.0 * scale);
        let player_rect = Rect::new(
            token.x - fallback.x * 0.5,
            token.y - fallback.y * PLAYER_FOOT_ANCHOR_Y,
            fallback.x,
            fallback.y,
        );
        draw_placeholder_sprite(player_rect, "FIGUR", PlaceholderCategory::Character, false);
    }

    fn draw_scene_background(&self, scene: &SceneMeta, rect: Rect) {
        let texture_key = self.scene_texture_key(scene.id);
        if let Some(texture) = self
            .scene_textures
            .get(texture_key)
            .or_else(|| self.scene_textures.get(scene.id))
        {
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, rust());
            return;
        }

        draw_room_placeholder(scene, rect);
    }

    fn draw_footer(&self, mouse: Vec2) {
        let footer = Rect::new(20.0, 590.0, 1240.0, 110.0);
        draw_rectangle(footer.x, footer.y, footer.w, footer.h, ui_dark());
        draw_rectangle_lines(footer.x, footer.y, footer.w, footer.h, 2.0, rust());

        self.draw_action_panel();

        draw_text_ex("INVENTAR", INVENTORY_X, 612.0, text_params(16, ochre()));
        for (index, item_id) in self.state.inventory.iter().enumerate() {
            if let Some(item) = item_meta(item_id) {
                let rect = inventory_slot_rect(index);
                draw_inventory_item(
                    item,
                    rect,
                    self.inventory_icons.as_ref(),
                    self.state.selected_item.as_deref() == Some(item.id),
                    rect.contains(mouse),
                );
            }
        }

        draw_footer_notebook(self);
    }

    fn draw_action_panel(&self) {
        draw_text_ex("VERB", 34.0, 612.0, text_params(16, ochre()));
        for (index, verb) in Verb::ALL.iter().copied().enumerate() {
            draw_verb_button(
                verb,
                verb_button_rect(index),
                self.state.verb == verb,
                self.verb_icons.as_ref(),
            );
        }
    }

    fn draw_dialogue(&self, id: &'static str, mouse: Vec2) {
        overlay();
        panel(Rect::new(130.0, 72.0, 1020.0, 560.0));
        button(dialogue_close_rect(), "X", false, mouse);
        if let Some(dialogue) = dialogue(id) {
            draw_text_ex(dialogue.speaker, 165.0, 124.0, text_params(28, bone()));

            let opening_rect = Rect::new(165.0, 146.0, 950.0, 62.0);
            draw_dialogue_text_box(opening_rect, dialogue.opening, muted());

            let response = self
                .dialogue_response
                .as_ref()
                .filter(|response| response.dialogue_id == id)
                .map(|response| response.text);
            if let Some(response) = response {
                let response_rect = Rect::new(165.0, 222.0, 950.0, 112.0);
                draw_rectangle(
                    response_rect.x,
                    response_rect.y,
                    response_rect.w,
                    response_rect.h,
                    Color::new(0.04, 0.07, 0.09, 0.95),
                );
                draw_rectangle_lines(
                    response_rect.x,
                    response_rect.y,
                    response_rect.w,
                    response_rect.h,
                    2.0,
                    amber(),
                );
                draw_text_ex(
                    "ANTWORT",
                    response_rect.x + 18.0,
                    response_rect.y + 25.0,
                    text_params(14, amber()),
                );
                draw_text_wrapped(
                    response,
                    response_rect.x + 18.0,
                    response_rect.y + 56.0,
                    response_rect.w - 36.0,
                    19.0,
                    paper(),
                );
            }

            let has_response = response.is_some();
            for (i, choice) in dialogue.choices.iter().enumerate() {
                draw_dialogue_choice(dialogue_choice_rect(i, has_response), choice.label, mouse);
            }
        }
    }

    fn draw_milestone(&self, mouse: Vec2) {
        overlay();
        panel(Rect::new(270.0, 155.0, 740.0, 410.0));
        centered_text("FALL 000", 640.0, 220.0, 18, amber());
        centered_text("Rettungslauf freigegeben", 640.0, 270.0, 34, bone());
        draw_text_wrapped(
            "Signalweg, Host-Karte, Bodenmarken, Stadtzeit, Lichtreferenz und Schimmelbrüder-Frame sind stabil. Die Regie kann den vermissten Host an die Gegenwart zurückziehen.",
            350.0,
            320.0,
            580.0,
            22.0,
            muted(),
        );
        button(
            button_rect(540.0, 492.0, 200.0, 48.0),
            "Weiter",
            true,
            mouse,
        );
    }

    fn hotspots(&self) -> Vec<&'static HotspotSpec> {
        current_scene(&self.state.scene)
            .hotspots
            .iter()
            .filter(|hotspot| self.hotspot_visible(hotspot))
            .collect()
    }

    fn hotspot_visible(&self, hotspot: &HotspotSpec) -> bool {
        match hotspot.id {
            "dispo_board" => !self.has_item("call_sheet"),
            "gaffer_roll" => !self.has_item("gaffer_tape"),
            "sdi_label_printer" => !self.has_item("sdi_label"),
            "print_shop" => {
                !self.has_item("lower_third_card")
                    || (!self.has_item("busker_sign")
                        && self.has_item("cardboard_piece")
                        && self.flag("city_notice_checked"))
            }
            "city_reflector" => !self.has_item("city_reflector"),
            "market_boxes" => !self.has_item("cardboard_piece"),
            "corner_kiosk" => !self.has_item("clear_tape"),
            "mold_token" => !self.has_item("mold_token"),
            "nurse_station" => self.flag("nurse_called"),
            "discharge_clipboard" => !self.has_item("medical_release"),
            "checksum_printout" => !self.has_item("checksum_note"),
            _ => true,
        }
    }

    fn hotspot(&self, id: &str) -> Option<&'static HotspotSpec> {
        current_scene(&self.state.scene)
            .hotspots
            .iter()
            .find(|hotspot| hotspot.id == id)
    }

    fn flag(&self, flag: &str) -> bool {
        self.state.flags.contains(flag)
    }

    fn has_item(&self, item: &str) -> bool {
        self.state.inventory.iter().any(|id| id == item)
    }

    fn scene_texture_key(&self, scene_id: &'static str) -> &'static str {
        match scene_id {
            "hospital_room" if self.has_item("medical_release") => "hospital_room_no_clipboard",
            "hospital_room" if self.flag("nurse_called") => "hospital_room_nurse",
            "archive_recovery" if self.has_item("checksum_note") => "archive_recovery_no_checksum",
            "video_schimmelbrueder" if self.has_item("mold_token") => {
                "video_schimmelbrueder_no_token"
            }
            _ => scene_id,
        }
    }

    fn add_item(&mut self, item: &str) {
        if !self.has_item(item) {
            self.state.inventory.push(item.to_string());
        }
    }

    fn take_item(&mut self, item: &str, flag: &str, status: &str, log: &str) {
        if self.flag(flag) || self.has_item(item) {
            if let Some(meta) = item_meta(item) {
                self.status = format!("{} ist bereits im Inventar.", meta.name);
            }
            return;
        }
        self.state.flags.insert(flag.to_string());
        self.add_item(item);
        self.add_log(log);
        self.status = status.to_string();
    }

    fn add_log(&mut self, line: &str) {
        if line.is_empty() || self.state.log.last().map(|s| s.as_str()) == Some(line) {
            return;
        }
        self.state.log.push(line.to_string());
        if self.state.log.len() > 40 {
            self.state.log.remove(0);
        }
    }

    fn set_flag_log(&mut self, flag: &str, log: &str, status: &str) {
        if !self.flag(flag) {
            self.state.flags.insert(flag.to_string());
            self.add_log(log);
        }
        self.status = status.to_string();
    }

    fn trigger_consequence(&mut self, kind: DeathKind, message: &'static str) {
        self.walk_target = None;
        self.state.selected_item = None;
        self.death = Some(DeathState {
            kind,
            started_at: runtime_time(),
        });
        self.status = message.to_string();
        self.add_log(message);
        save_state(&self.state);
    }

    fn complete_consequence_transition(&mut self) {
        let Some(consequence) = self.death.take() else {
            return;
        };
        self.state.scene = consequence.kind.destination().to_string();
        self.state.selected_item = None;
        self.modal = Modal::None;
        self.dialogue_response = None;
        self.status = consequence.kind.arrival_status().to_string();
        self.add_log(consequence.kind.arrival_status());
        self.reset_player_position();
        save_state(&self.state);
    }

    fn travel(&mut self, scene: &str, status: &str) {
        if SCENES.iter().all(|candidate| candidate.id != scene) {
            self.status = "Dieser Weg ist noch nicht vorbereitet.".to_string();
            return;
        }
        self.state.scene = scene.to_string();
        self.state.selected_item = None;
        self.modal = Modal::None;
        self.death = None;
        self.status = status.to_string();
        self.reset_player_position();
        save_state(&self.state);
    }

    fn reset(&mut self) {
        clear_save();
        self.state = GameState::default();
        self.status = "Fall 000 neu gestartet. Die Fallakte liegt wieder am Anfang.".to_string();
        self.modal = Modal::None;
        self.dialogue_response = None;
        self.death = None;
        self.reset_player_position();
    }
}

fn current_scene(id: &str) -> &'static SceneMeta {
    SCENES
        .iter()
        .find(|scene| scene.id == id)
        .unwrap_or(&SCENES[0])
}

fn dialogue(id: &str) -> Option<&'static Dialogue> {
    DIALOGUES.iter().find(|dialogue| dialogue.id == id)
}

fn item_meta(id: &str) -> Option<&'static ItemMeta> {
    ITEMS.iter().find(|item| item.id == id)
}

fn pickup_target_name(id: &str) -> Option<&'static str> {
    match id {
        "dispo_board" => Some("Fallakte 000"),
        "gaffer_roll" => Some("Gaffer-Tape"),
        "sdi_label_printer" => Some("SDI-Label"),
        "print_shop" => Some("Host-Karte"),
        "city_reflector" => Some("Kofferreflektor"),
        "market_boxes" => Some("Pappstück"),
        "corner_kiosk" => Some("Transparentband"),
        "mold_token" => Some("Frame-Probe"),
        "discharge_clipboard" => Some("Entlassungsbogen"),
        "checksum_printout" => Some("Prüfsummenzettel"),
        _ => None,
    }
}

fn funny_item_mismatch(hotspot_id: &str, item_id: &str) -> Option<&'static str> {
    match (hotspot_id, item_id) {
        ("nurse_station", "mold_token") => {
            Some("Die Pflegekraft akzeptiert keine Frame-Probe als zweiten Puls. Immerhin ist sie stabiler als deine Entscheidung.")
        }
        ("hospital_monitor", "medical_release") => {
            Some("Der Monitor liest den Entlassungsbogen nicht ein. Er piept nur so, als hätte er kurz darüber nachgedacht.")
        }
        ("discharge_clipboard", "clear_tape") => {
            Some("Das Formular ist schon an ein Klemmbrett gebunden. Mehr Klebeband wäre Verwaltungskunst.")
        }
        ("busker_case", "mold_token") => {
            Some("Der Musiker sagt, seine Koffer hatten schon viel gesehen, aber keine freiwillige Frame-Probe.")
        }
        ("print_shop", "medical_release") => {
            Some("Der Copyshop kann den Entlassungsbogen laminieren. Das würde ihn nicht gültiger machen, nur unangenehmer.")
        }
        ("graphics_terminal", "mold_token") => {
            Some("Der Grafikplatz erkennt die Frame-Probe als Bildmaterial, das ausdrücklich nicht in eine Host-Karte gehört.")
        }
        ("station_clock", "sdi_label") => {
            Some("Das SDI-Label an die Bahnhofsuhr zu kleben würde die Zeit nicht routen, nur Hausmeister wecken.")
        }
        ("water_tower", "clear_tape") => {
            Some("Transparentband gegen Backstein und Tageslicht: mutig, aber nicht lichttechnisch.")
        }
        ("video_hub", "mold_token") => {
            Some("Die Frame-Probe gehört in den Clip. Die Kreuzschiene ist schon mit normalen Signalen beleidigt genug.")
        }
        ("klixx_table", "medical_release") => {
            Some("Der Entlassungsbogen auf dem Klixx-Tisch wirkt wie ein sehr defensiver Tippzettel.")
        }
        ("record_store", "sdi_label") => {
            Some("Das SDI-Label passt auf keine Platte. Der Verkäufer würde es trotzdem nach Genre sortieren.")
        }
        ("corner_kiosk", "medical_release") => {
            Some("Der Kiosk akzeptiert keine Entlassungsbögen als Zahlungsmittel. Das Gesundheitssystem auch nicht.")
        }
        ("greenscreen_setup", "busker_sign") => {
            Some("Das Kofferschild im Studio würde nur beweisen, dass der Straßenmusiker nicht hier ist.")
        }
        ("song_exit_gate", "call_sheet") => {
            Some("Die Fallakte erklärt den Ausgang. Sie öffnet ihn nicht. Bürokratie hat Grenzen, auch im Archiv.")
        }
        _ => None,
    }
}

fn poke_response(id: &str, name: &str) -> String {
    match id {
        "dispo_board" => "Die Falltafel wackelt. Die Akte bleibt genau dort, wo man sie eigentlich aufheben sollte.".to_string(),
        "route_map" => "Der Plan knistert. Gebäude 9, 11 und 15 rücken nicht näher zusammen, aber die rote Route sieht jetzt genervt aus.".to_string(),
        "equipment_storage" => "Du stubst die Lagertür an. Dahinter antwortet ein Regal mit sehr vielen Dingen, die nicht deine Aufgabe sind.".to_string(),
        "staircase" => "Die Treppe bleibt Treppe. Sie hat seit Jahren Erfahrung darin, Leute ohne Drama hoch und runter zu lassen.".to_string(),
        "loading_zone" => "Eine Rollkiste klackt. Der Hof klingt kurz wie ein sehr kleines Schlagzeug.".to_string(),
        "address_plate" => "Das Schild gibt ein trockenes Metallgeräusch von sich. Die Adresse bleibt korrekt.".to_string(),
        "greenscreen_wall" => "Der Stoff federt zurück. Für einen Moment sieht die ganze Wand aus, als würde sie dich beurteilen.".to_string(),
        "floor_marks" => "Die alten Marken lösen sich nicht. Sie wissen, wie man auf einem Studioboden überlebt.".to_string(),
        "klixx_table" => "Der Tisch ruckt kaum. Jemand hat ihn offenbar schon gegen genau solche Praktikanten gesichert.".to_string(),
        "chat_preview" => "Der Chatpreview springt nicht an. Die Kommentare bleiben als potenzielle Bedrohung gespeichert.".to_string(),
        "camera_one" => "Die Kamera schwingt minimal nach. Die Regie hätte das gesehen, wenn sie dich nicht schon kennen würde.".to_string(),
        "mentor_shadow" | "intercom_voice" => "Die Gegensprechanlage knackt. Eine Stimme sagt: \"Bitte keine haptische Kommunikation mit der Regie.\"".to_string(),
        "greenscreen_setup" => "Das Setup nimmt den Stoß als Regieimpuls, verwirft ihn und bleibt ungerettet.".to_string(),
        "rehearsal_monitor" => "Der Monitor flackert. Für einen Moment steht der Host-Schatten an zwei Orten und entscheidet sich für keinen.".to_string(),
        "mixing_console" => "Ein Fader schnippt zurück. Die Regie klingt danach genau null Prozent geretteter.".to_string(),
        "video_schimmelbrueder" => "Der Schimmelbrüder-Frame flackert zurück. Das Bild fühlt sich tiefer an, als ein Monitor sollte.".to_string(),
        "casting_table" => "Der Gießtisch antwortet dumpf. In der Formenhalle klingt selbst ein Fehler industriell.".to_string(),
        "mold_token" => "Die Frame-Probe klickt leicht gegen den Boden. Irgendwo in der Regie zuckt eine Pegelanzeige.".to_string(),
        "patterned_belt" => "Das Muster bleibt stur. Es ist ein Rätsel, kein Teppich.".to_string(),
        "factory_floor" => "Der Hallenboden nimmt den Stoß kommentarlos. Er hat größere Maschinen überlebt.".to_string(),
        "song_exit_gate" => "Der Clip-Ausgang summt. Er will keine Gewalt, sondern eine sauber zugeordnete Probe.".to_string(),
        "alarm_panel" => "Das Alarmfeld klickt. Mehr Alarm als Alarm wäre organisatorisch schwierig.".to_string(),
        "extinguisher_cabinet" => "Der Löschschrank scheppert vollständig. Genau das sollte er beweisen.".to_string(),
        "safety_officer" => "Der Sicherheitsdienst notiert: \"Stupst Dinge an.\" Du bekommst keinen Durchschlag.".to_string(),
        "fire_return_door" => "Die nasse Tür schlägt nicht zurück. Arbeitsschutz ist manchmal enttäuschend unspektakulär.".to_string(),
        "archive_drive" => "Das Archivlaufwerk klackt synchron zurück. Immerhin habt ihr jetzt einen gemeinsamen Takt.".to_string(),
        "recovery_terminal" => "Das Terminal nimmt Stöße nicht als Prüfsumme. Es ist alt, aber nicht verzweifelt.".to_string(),
        "checksum_printout" => "Der Ausdruck rutscht einen Millimeter. Die Prüfsumme bleibt beleidigend exakt.".to_string(),
        "control_room_return" => "Die Tür zur Regie bleibt offen genug. Mehr Überzeugung braucht sie nicht.".to_string(),
        "street_mural" => "Die Wand klingt massiv. Kunstkritik per Knöchel wird hier nicht archiviert.".to_string(),
        "city_notice" => "Der Aushang flattert. Der Hinweis auf den Kofferreflektor bleibt dran.".to_string(),
        "platform_sign" => "Das Schild wippt nicht. Es hat Pendler überlebt und ist entsprechend abgestumpft.".to_string(),
        "public_phone" => "Die Münzrückgabe klackt. Das Telefon spendet nichts außer einem historischen Geräusch.".to_string(),
        "station_clock" => "Die Uhr lässt sich nicht beeindrucken. Zeitreferenzen mögen keine körperliche Nähe.".to_string(),
        "water_tower" => "Der Wasserturm reagiert nicht. Aus Sicht des Turms war das wahrscheinlich Wetter.".to_string(),
        "city_reflector" => "Der festgeschraubte Parkreflektor klickt nur trocken. Mobil ist hier nichts.".to_string(),
        "tv_tower_view" => "Du stubst in die Richtung des Fernsehturms. Hamburg bleibt an Ort und Stelle.".to_string(),
        "print_shop" => "Der Tresen klackt. Der Copyshop druckt trotzdem nur Dinge, nicht Gefühle.".to_string(),
        "market_boxes" => "Die Kisten rutschen gerade genug, um ein brauchbares Pappstück zu verraten.".to_string(),
        "cable_bin" => "Die nassen Kabelreste klatschen gegen die Tonne. Niemand wirkt dadurch sicherer.".to_string(),
        "paint_cans" => "Eine Farbdose rollt einen Zentimeter und entscheidet sich dann gegen weitere Handlung.".to_string(),
        "brace_beam" | "release_rope" | "workshop_gap" | "collapse_exit" => "Das blockierte Material antwortet mit einem Geräusch, das nach Versicherung klingt.".to_string(),
        _ => format!("Du stubst {name} an. Es passiert etwas Messbares, aber leider nichts Nützliches."),
    }
}

fn tongue_response(id: &str, name: &str) -> String {
    match id {
        "dispo_board" => "Die Fallakte schmeckt nach Filzstift, Panik und einem Zeitstempel, der nicht ganz trocken ist.".to_string(),
        "route_map" => "Der Gebäudeplan schmeckt nach Laminat. Haus 11 bleibt trotzdem unangenehm nah.".to_string(),
        "greenscreen_wall" | "floor_marks" | "gaffer_roll" | "chat_preview" => "Produktionsmaterial mit der Zunge zu prüfen bleibt die schnellste Route zur Evakuierung.".to_string(),
        "camera_one" => "Die Kamera hat keine Nahgrenze für Zungen. Die Regie leider auch nicht.".to_string(),
        "mixing_console" => "Das Mischpult bleibt unangetastet. Deine Zunge hat heute schon genug Produktionswerte gefährdet.".to_string(),
        "video_schimmelbrueder" => "Der Monitor schmeckt nach Staub und schlechtem Timing. Der Frame dahinter wirkt trotzdem begehbar.".to_string(),
        "casting_table" => "Der Gießtisch schmeckt nach Metall und einer sehr langen Sicherheitsunterweisung.".to_string(),
        "mold_token" => "Die Frame-Probe schmeckt nach feuchtem Material und einer Entscheidung, die nicht ins Protokoll kommt.".to_string(),
        "patterned_belt" => "Das Muster mit der Zunge zu lesen zählt nicht als Mustererkennung.".to_string(),
        "factory_floor" => "Der Hallenboden beantwortet die Frage, ob alte Industrie schmeckt: ja, nach nein.".to_string(),
        "song_exit_gate" => "Der Clip-Ausgang akzeptiert keine biometrische Zungenprüfung.".to_string(),
        "alarm_panel" | "extinguisher_cabinet" => "Brandschutztechnik wird nicht besser, wenn man sie anzüngelt. Sie wird nur persönlicher.".to_string(),
        "safety_officer" => "Der Sicherheitsdienst sagt: \"Ich schreibe das nicht auf, weil dann jemand es lesen müsste.\"".to_string(),
        "archive_drive" | "recovery_terminal" | "checksum_printout" => "Archivtechnik schmeckt nach altem Staub und neuer Reue.".to_string(),
        "street_mural" => "Die Wand schmeckt nach Regen und sehr alter Farbe. Keine der Schichten gibt Questhinweise frei.".to_string(),
        "city_notice" => "Der Aushang schmeckt nach Laternenmast. Der Hinweis wäre durch Lesen schneller gewesen.".to_string(),
        "public_phone" => "Der Hörer schmeckt nach Altplastik und Entscheidungen aus Zeiten vor Touchscreens.".to_string(),
        "station_clock" | "platform_sign" => "Öffentliche Infrastruktur ist nicht zum Probieren gedacht. Das erklärt einiges an Beschilderung.".to_string(),
        "water_tower" | "tv_tower_view" => "Die Distanz verhindert das Schlimmste. Ausnahmsweise arbeitet Perspektive für dich.".to_string(),
        "print_shop" => "Der Copyshop schmeckt schon von außen nach Toner. Das reicht als Diagnose.".to_string(),
        "record_store" => "Vinylkultur mit der Zunge zu prüfen ist genau der Grund, warum manche Läden Schilder brauchen.".to_string(),
        "cable_bin" => "Nasser Kabelmüll bleibt unprobiert. Der Arbeitsschutz bekommt ausnahmsweise gute Nachrichten.".to_string(),
        "paint_cans" => "Die Farbdosen bleiben geschlossen. Geschmackstests an Bühnenfarbe sind kein Abkürzungs-Genre.".to_string(),
        _ => format!("{name} bleibt nach kurzer Prüfung offiziell unangezüngelt. Das ist für alle besser."),
    }
}

fn smell_response(id: &str, name: &str) -> String {
    match id {
        "dispo_board" => "Es riecht nach Filzstift, Papier und Entscheidungen, die jemand anders getroffen hat.".to_string(),
        "route_map" => "Der Gebäudeplan riecht nach Laminat und alten Wegen. Der rote Faden ist leider geruchlos.".to_string(),
        "klixx_table" => "Der Tisch riecht nach Studio, kaltem Kaffee und der Sorte Spannung, die kurz vor einer Auflösung entsteht.".to_string(),
        "gaffer_roll" | "floor_marks" => "Es riecht nach Kleber, Staub und einem Boden, der schon bessere Takes gesehen hat.".to_string(),
        "server_racks" | "sdi_label_printer" | "sdi_spool" | "video_hub" => "Es riecht nach warmem Plastik und Kabeln, die so tun, als wären sie beschriftet.".to_string(),
        "rehearsal_monitor" | "video_schimmelbrueder" => "Der Monitor riecht nach Elektronik. Der geöffnete Frame dahinter riecht nach feuchter Fertigungshalle.".to_string(),
        "mixing_console" => "Das Mischpult riecht nach warmen Fadern, Kaffee und Entscheidungen, die live niemand erklären will.".to_string(),
        "casting_table" | "factory_floor" => "Die Halle riecht nach Metall, Formmasse und einem Song, der zu lange im Raum stand.".to_string(),
        "patterned_belt" => "Die Musterbahn riecht neutral. Sie verlässt sich auf Augen, nicht Nasen.".to_string(),
        "song_exit_gate" => "Der Ausgang riecht nach kaltem Strom und einem Schnittpunkt im Material.".to_string(),
        "alarm_panel" | "extinguisher_cabinet" => "Es riecht nach nasser Wand, Löschschrank und Formularen mit drei Durchschlägen.".to_string(),
        "safety_officer" => "Der Sicherheitsdienst riecht nach Regenjacke, Kaffee und berechtigter Skepsis.".to_string(),
        "archive_drive" | "recovery_terminal" => "Es riecht nach Magnetband, Staub und einem Backup, das zu spät ernst genommen wurde.".to_string(),
        "checksum_printout" => "Der Ausdruck riecht nach warmem Toner. Das ist die sinnlichste Prüfsumme, die du bekommst.".to_string(),
        "city_notice" | "street_mural" => "Es riecht nach Straße, Kleister und Information, die lieber gelesen werden möchte.".to_string(),
        "station_clock" | "platform_sign" => "Es riecht nach Bahnhof: Bremsstaub, Kaffee und exakt genug Zeitdruck.".to_string(),
        "public_phone" => "Das Telefon riecht nach Regen, Metall und Gesprächsabbrüchen aus einer anderen Zeit.".to_string(),
        "water_tower" => "Das Parklicht riecht nicht, aber die Luft dort erklärt, warum die Referenz brauchbar ist.".to_string(),
        "print_shop" => "Der Copyshop riecht nach Papierstapel und der stillen Macht eines Schneidehebels.".to_string(),
        "market_boxes" => "Die Kisten riechen nach Karton und Gemüsevergangenheit.".to_string(),
        "cable_bin" => "Die Tonne riecht nach nassem PVC und der Sorte Reparatur, die niemand mehr versucht.".to_string(),
        "paint_cans" => "Die Farbdosen riechen nach Kulisse, Lösungsmittel und sehr kurzer Belüftungsplanung.".to_string(),
        _ => format!("{name} riecht nach Umgebung. Für einen Questhinweis ist das zu ehrlich."),
    }
}

fn use_response(id: &str, name: &str) -> String {
    match id {
        "dispo_board" => "Die Tafel ist kein Gerät. Der brauchbare Teil ist die Fallakte, und die will aufgehoben werden.".to_string(),
        "route_map" => "Der Plan lässt sich nicht benutzen. Er ist bereits das Maximum an Hilfe, das Papier leisten kann.".to_string(),
        "rehearsal_monitor" => "Der Anker-Monitor wartet auf den stabilisierten Schimmelbrüder-Frame und auf vollständige Studio-Referenzen.".to_string(),
        "mixing_console" => "Das Mischpult mischt Kanäle, keine Rettungslogik. Die wichtigen Anker bleiben Monitor, Grafikplatz und Studio-Rückweg.".to_string(),
        "on_air_lamp" => "Die Lampe bleibt aus. Sie geht erst an, wenn die Rettung nicht mehr nach Praktikanten-Experiment aussieht.".to_string(),
        "casting_table" | "mold_rack" | "patterned_belt" | "factory_floor" => "Die Fertigungshalle will keine Bedienung, sondern Beobachtung: Boden prüfen, Probe nehmen, Muster verstehen.".to_string(),
        "mold_token" => "Die Frame-Probe ist kein Schalter. Sie muss an Formenreihe, Musterbahn und Clip-Ausgang Sinn ergeben.".to_string(),
        "city_notice" => "Der Aushang lässt sich nicht benutzen. Lesen ist hier bereits die Interaktion mit dem höchsten Budget.".to_string(),
        "public_phone" => "Das Telefon hat kein Freizeichen. Selbst die Vergangenheit geht nicht ran.".to_string(),
        "record_store" => "Der Plattenladen ist keine Lösung, nur Atmosphäre mit Preisschildern.".to_string(),
        "market_boxes" => "Die Kisten sind nur nützlich, wenn du das Pappstück aufhebst.".to_string(),
        "corner_kiosk" => "Der Kiosk hilft nicht auf Knopfdruck. Das Transparentband ist die eigentliche Pointe.".to_string(),
        "station_clock" => "Die Uhr benutzt dich bereits als Beobachter. Mehr Bedienung erlaubt sie nicht.".to_string(),
        "water_tower" => "Der Wasserturm ist als Lichtreferenz nützlich, nicht als Gerät. Schau genau hin oder notiere ihn in der Fallakte.".to_string(),
        "cable_bin" => "Die Kabeltonne ist Entsorgung, keine Ersatzteilquelle. Der Rückweg braucht das Brandschutzprotokoll.".to_string(),
        "paint_cans" => "Farbe löst keine Blockade. Strebe sichern, Riegel lösen, dann raus.".to_string(),
        "safety_officer" | "busker_case" | "nurse_station" | "mentor_shadow" | "intercom_voice" => "Das ist ein Mensch, kein Interface. Reden ist hier ausnahmsweise die moderne Lösung.".to_string(),
        _ => format!("{name} hat keine sinnvolle Benutzen-Funktion. Nicht alles mit Rand ist ein Schalter."),
    }
}

fn pickup_response(id: &str, name: &str) -> String {
    match id {
        "video_schimmelbrueder" => "Du kannst keinen Monitor aufheben, während du gerade planst, in ihn hineinzusteigen.".to_string(),
        "rehearsal_monitor" | "graphics_terminal" | "mixing_console" | "video_hub" | "server_racks" => "Zu schwer, zu angeschlossen und zu teuer, um es in die Inventarleiste zu stopfen.".to_string(),
        "safety_officer" | "nurse_station" | "busker_case" | "mentor_shadow" | "intercom_voice" => "Menschen gehören nicht ins Inventar. Diese Regel wurde nach langen Tests eingeführt.".to_string(),
        "water_tower" | "tv_tower_view" => "Du hebst kurz die Augen. Das muss reichen.".to_string(),
        "station_clock" => "Du kannst die Uhrzeit mitnehmen, aber nicht die Uhr. Notieren zählt in diesem Fall.".to_string(),
        "public_phone" => "Das Telefon ist fest montiert. Außerdem will niemand erklären, warum du ein öffentliches Telefon in der Tasche hast.".to_string(),
        "record_store" => "Einen ganzen Plattenladen mitzunehmen wäre logistisch stark, aber dramaturgisch unnötig.".to_string(),
        "cable_bin" => "Nasser Kabelmüll ist technisch gesehen tragbar, aber nur, wenn man alle Lebensentscheidungen ignoriert.".to_string(),
        "paint_cans" => "Die Farbdosen sind schwer, nass und exakt nicht das Werkzeug für diese Blockade.".to_string(),
        _ => format!("{name} ist kein loser Gegenstand. Deine Taschen sind ehrgeizig, aber nicht unbegrenzt."),
    }
}

fn talk_response(id: &str, name: &str) -> String {
    match id {
        "call_button" => "Der Rufknopf hat kein Gegensprechmodul. Er sagt nichts, aber er petzt zuverlässig.".to_string(),
        "hospital_monitor" => "Der Monitor antwortet in Pieptönen. Die Übersetzung lautet vermutlich: Bitte nicht.".to_string(),
        "rehearsal_monitor" | "video_schimmelbrueder" => "Der Monitor rauscht. Wenn er antwortet, ist das wahrscheinlich schon der falsche Teil der Geschichte.".to_string(),
        "mold_token" => "Die Frame-Probe sagt nichts. Das ist beruhigend, weil du sonst ein viel größeres Formular bräuchtest.".to_string(),
        "brass_players" => "Die Bläsergruppe antwortet gleichzeitig. Es klingt wie ein Akkord, aber die Aussage ist klar: erst auf die Eins, dann zurück zur Regie.".to_string(),
        "station_clock" => "Die Uhr redet nicht. Sie macht ihre Aussage pro Minute einmal.".to_string(),
        "record_store" => "Durch die Scheibe ist niemand gesprächsbereit. Das Urteil über deinen Geschmack findet intern statt.".to_string(),
        "water_tower" | "tv_tower_view" => "Die Skyline antwortet nicht. Hamburg ist in dieser Beziehung stabil.".to_string(),
        _ => format!("{name} hat keine Gesprächsebene. Du bekommst immerhin keine Widerrede."),
    }
}

fn hotspot_action_name(verb: Verb, hotspot: &HotspotSpec) -> &'static str {
    if verb == Verb::PickUp {
        pickup_target_name(hotspot.id).unwrap_or(hotspot.name)
    } else {
        hotspot.name
    }
}

fn is_exit_like(id: &str) -> bool {
    matches!(
        id,
        "hospital_exit"
            | "fire_return_door"
            | "collapse_exit"
            | "workshop_gap"
            | "control_room_return"
    )
}

fn save_state(state: &GameState) {
    if let Ok(json) = serde_json::to_string(state) {
        storage_set(SAVE_KEY, &json);
    }
}

fn load_state() -> GameState {
    storage_get(SAVE_KEY)
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn clear_save() {
    storage_remove(SAVE_KEY);
}

#[cfg(test)]
fn runtime_time() -> f64 {
    0.0
}

#[cfg(not(test))]
fn runtime_time() -> f64 {
    get_time()
}

#[cfg(target_arch = "wasm32")]
fn hide_loading_overlay() {
    unsafe {
        klixx_loading_complete();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn hide_loading_overlay() {}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn klixx_storage_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn macroquad_audio_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn quad_net_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn sapp_jsutils_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn klixx_storage_get_len(key_ptr: *const u8, key_len: u32) -> u32;
    fn klixx_storage_get(key_ptr: *const u8, key_len: u32, out_ptr: *mut u8, out_len: u32) -> u32;
    fn klixx_storage_set(key_ptr: *const u8, key_len: u32, value_ptr: *const u8, value_len: u32);
    fn klixx_storage_remove(key_ptr: *const u8, key_len: u32);
    fn klixx_loading_complete();
}

#[cfg(target_arch = "wasm32")]
fn storage_get(key: &str) -> Option<String> {
    let len = unsafe { klixx_storage_get_len(key.as_ptr(), key.len() as u32) };
    if len == u32::MAX {
        return None;
    }

    let mut bytes = vec![0; len as usize];
    let copied = unsafe {
        klixx_storage_get(
            key.as_ptr(),
            key.len() as u32,
            bytes.as_mut_ptr(),
            bytes.len() as u32,
        )
    };
    bytes.truncate(copied as usize);
    String::from_utf8(bytes).ok()
}

#[cfg(target_arch = "wasm32")]
fn storage_set(key: &str, value: &str) {
    unsafe {
        klixx_storage_set(
            key.as_ptr(),
            key.len() as u32,
            value.as_ptr(),
            value.len() as u32,
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn storage_remove(key: &str) {
    unsafe {
        klixx_storage_remove(key.as_ptr(), key.len() as u32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
const NATIVE_SAVE_FILE: &str = "klixx.local-save-v11.json";

#[cfg(not(target_arch = "wasm32"))]
fn storage_get(_key: &str) -> Option<String> {
    std::fs::read_to_string(NATIVE_SAVE_FILE).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_set(_key: &str, value: &str) {
    let _ = std::fs::write(NATIVE_SAVE_FILE, value);
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_remove(_key: &str) {
    let _ = std::fs::remove_file(NATIVE_SAVE_FILE);
}

fn virtual_mouse() -> Vec2 {
    let (mx, my) = mouse_position();
    vec2(mx / screen_width() * VW, my / screen_height() * VH)
}

fn pct_rect(scene: Rect, pct: Rect) -> Rect {
    Rect::new(
        scene.x + scene.w * pct.x / 100.0,
        scene.y + scene.h * pct.y / 100.0,
        scene.w * pct.w / 100.0,
        scene.h * pct.h / 100.0,
    )
}

fn pct_point(scene: Rect, point: (f32, f32)) -> Vec2 {
    vec2(
        scene.x + scene.w * point.0 / 100.0,
        scene.y + scene.h * point.1 / 100.0,
    )
}

fn hotspot_polygon(scene: &SceneMeta, hotspot: &HotspotSpec) -> Option<&'static [(f32, f32)]> {
    HOTSPOT_POLYGONS
        .iter()
        .find(|polygon| {
            polygon.scene_id == scene.id
                && polygon.hotspot_id == hotspot.id
                && polygon.points.len() >= 3
        })
        .map(|polygon| polygon.points)
}

fn hotspot_polygon_points(
    scene: &SceneMeta,
    hotspot: &HotspotSpec,
    rect: Rect,
) -> Option<Vec<Vec2>> {
    hotspot_polygon(scene, hotspot)
        .map(|points| points.iter().map(|point| pct_point(rect, *point)).collect())
}

fn hotspot_contains(scene: &SceneMeta, hotspot: &HotspotSpec, rect: Rect, point: Vec2) -> bool {
    if let Some(polygon) = hotspot_polygon_points(scene, hotspot, rect) {
        return point_in_polygon(point, &polygon);
    }

    pct_rect(rect, hotspot.pct).contains(point)
}

fn hovered_hotspot(
    scene: &SceneMeta,
    hotspots: &[&'static HotspotSpec],
    rect: Rect,
    point: Vec2,
) -> Option<&'static HotspotSpec> {
    let mut best: Option<(&'static HotspotSpec, i32, f32)> = None;

    for &hotspot in hotspots {
        if !hotspot_contains(scene, hotspot, rect, point) {
            continue;
        }

        let priority = hotspot_priority(hotspot.kind);
        let area = hotspot_area(scene, hotspot, rect);
        let beats_best = best
            .map(|(_, best_priority, best_area)| {
                priority > best_priority
                    || (priority == best_priority && area < best_area - f32::EPSILON)
            })
            .unwrap_or(true);

        if beats_best {
            best = Some((hotspot, priority, area));
        }
    }

    best.map(|(hotspot, _, _)| hotspot)
}

fn hotspot_priority(kind: HotspotKind) -> i32 {
    match kind {
        HotspotKind::Pickup => 40,
        HotspotKind::Character => 30,
        HotspotKind::Exit => 20,
        HotspotKind::Prop => 10,
    }
}

fn hotspot_area(scene: &SceneMeta, hotspot: &HotspotSpec, rect: Rect) -> f32 {
    if let Some(points) = hotspot_polygon(scene, hotspot) {
        let mut area = 0.0;
        for index in 0..points.len() {
            let a = points[index];
            let b = points[(index + 1) % points.len()];
            area += a.0 * b.1 - b.0 * a.1;
        }
        return area.abs() * 0.5 * rect.w * rect.h / 10_000.0;
    }

    let bounds = pct_rect(rect, hotspot.pct);
    bounds.w * bounds.h
}

fn hotspot_bounds(scene: &SceneMeta, hotspot: &HotspotSpec, rect: Rect) -> Rect {
    let Some(polygon) = hotspot_polygon_points(scene, hotspot, rect) else {
        return pct_rect(rect, hotspot.pct);
    };

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in polygon {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn draw_hotspot_hover(scene: &SceneMeta, hotspot: &HotspotSpec, rect: Rect) {
    if let Some(polygon) = hotspot_polygon_points(scene, hotspot, rect) {
        for index in 0..polygon.len() {
            let a = polygon[index];
            let b = polygon[(index + 1) % polygon.len()];
            draw_line(a.x, a.y, b.x, b.y, 1.5, ochre());
        }
        return;
    }

    let r = pct_rect(rect, hotspot.pct);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, ochre());
}

fn constrain_to_walkable(target: Vec2, scene: &SceneMeta, rect: Rect) -> Vec2 {
    if scene.walkable.len() < 3 {
        return target;
    }

    let polygon: Vec<Vec2> = scene
        .walkable
        .iter()
        .map(|point| pct_point(rect, *point))
        .collect();

    if point_in_polygon(target, &polygon) {
        return target;
    }

    closest_point_on_polygon(target, &polygon)
}

fn player_depth_scale(point: Vec2, scene: &SceneMeta, rect: Rect) -> f32 {
    if scene.walkable.len() < 3 {
        return PLAYER_DEPTH_MAX_SCALE;
    }

    let (min_y, max_y) = scene
        .walkable
        .iter()
        .map(|point| pct_point(rect, *point).y)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min_y, max_y), y| {
            (min_y.min(y), max_y.max(y))
        });
    let span = max_y - min_y;
    if !span.is_finite() || span < 8.0 {
        return PLAYER_DEPTH_MAX_SCALE;
    }

    let depth = ((point.y - min_y) / span).clamp(0.0, 1.0);
    PLAYER_DEPTH_MIN_SCALE + (PLAYER_DEPTH_MAX_SCALE - PLAYER_DEPTH_MIN_SCALE) * depth
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut previous = polygon.len() - 1;

    for current in 0..polygon.len() {
        let a = polygon[current];
        let b = polygon[previous];
        if (closest_point_on_segment(point, a, b) - point).length_squared() <= 1.0 {
            return true;
        }
        if (a.y > point.y) != (b.y > point.y) {
            let crossing_x = (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
            if point.x < crossing_x {
                inside = !inside;
            }
        }
        previous = current;
    }

    inside
}

fn closest_point_on_polygon(point: Vec2, polygon: &[Vec2]) -> Vec2 {
    let mut closest = polygon[0];
    let mut best_distance = (point - closest).dot(point - closest);

    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let candidate = closest_point_on_segment(point, a, b);
        let distance = (point - candidate).dot(point - candidate);
        if distance < best_distance {
            best_distance = distance;
            closest = candidate;
        }
    }

    closest
}

fn closest_point_on_segment(point: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let segment = b - a;
    let length_squared = segment.dot(segment);
    if length_squared <= f32::EPSILON {
        return a;
    }

    let t = ((point - a).dot(segment) / length_squared).clamp(0.0, 1.0);
    a + segment * t
}

fn button_rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(x, y, w, h)
}

fn dialogue_close_rect() -> Rect {
    button_rect(1088.0, 88.0, 42.0, 42.0)
}

fn dialogue_choice_rect(index: usize, has_response: bool) -> Rect {
    if has_response {
        button_rect(165.0, 354.0 + index as f32 * 52.0, 950.0, 44.0)
    } else {
        button_rect(165.0, 295.0 + index as f32 * 58.0, 950.0, 52.0)
    }
}

fn scene_rect() -> Rect {
    Rect::new(20.0, 92.0, 1240.0, 480.0)
}

fn scene_token_position(scene: &SceneMeta) -> Vec2 {
    let rect = scene_rect();
    vec2(
        rect.x + rect.w * scene.token.0 / 100.0,
        rect.y + rect.h * scene.token.1 / 100.0,
    )
}

fn facing_from_delta(delta: Vec2) -> PlayerFacing {
    if delta.x.abs() > delta.y.abs() {
        if delta.x < 0.0 {
            PlayerFacing::Left
        } else {
            PlayerFacing::Right
        }
    } else if delta.y < 0.0 {
        PlayerFacing::Up
    } else {
        PlayerFacing::Down
    }
}

fn walk_cycle_frame(time: f64) -> f32 {
    const CYCLE: [f32; 4] = [1.0, 2.0, 3.0, 2.0];
    CYCLE[((time * 8.0) as usize) % CYCLE.len()]
}

fn inventory_slot_rect(index: usize) -> Rect {
    let col = index % INVENTORY_COLUMNS;
    let row = index / INVENTORY_COLUMNS;
    button_rect(
        INVENTORY_X + col as f32 * (INVENTORY_SLOT + INVENTORY_GAP),
        INVENTORY_Y + row as f32 * (INVENTORY_SLOT + INVENTORY_GAP),
        INVENTORY_SLOT,
        INVENTORY_SLOT,
    )
}

fn verb_button_rect(index: usize) -> Rect {
    let col = index % 4;
    let row = index / 4;
    button_rect(
        34.0 + col as f32 * 50.0,
        620.0 + row as f32 * 36.0,
        44.0,
        32.0,
    )
}

#[derive(Clone, Copy)]
enum PlaceholderCategory {
    Character,
    Prop,
    Bg,
    Ui,
}

impl PlaceholderCategory {
    fn colors(self) -> (Color, Color) {
        match self {
            PlaceholderCategory::Character => (col(0x7a, 0x8f, 0x55), WHITE),
            PlaceholderCategory::Prop => (col(0x4a, 0x3f, 0x35), WHITE),
            PlaceholderCategory::Bg => (paper_black(), col(0xa8, 0x9a, 0x82)),
            PlaceholderCategory::Ui => (ui_mid(), paper()),
        }
    }
}

fn draw_room_placeholder(scene: &SceneMeta, rect: Rect) {
    if scene.id == "office_hall" {
        draw_office_hall_room(rect);
        return;
    }
    if scene.id == "building_courtyard" {
        draw_courtyard_room(rect);
        return;
    }
    if scene.id == "server_room" {
        draw_server_room(rect);
        return;
    }
    if scene.id == "set_workshop" {
        draw_set_workshop_room(rect);
        return;
    }

    draw_placeholder_sprite(
        rect,
        &format!("RAUM\n{}\n{}", scene.zone.to_uppercase(), scene.name),
        PlaceholderCategory::Bg,
        false,
    );
}

fn draw_office_hall_room(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col(0x1f, 0x1c, 0x18));
    draw_rectangle(rect.x, rect.y, rect.w, rect.h * 0.62, col(0x33, 0x2d, 0x26));
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.62,
        rect.w,
        rect.h * 0.38,
        col(0x18, 0x17, 0x15),
    );
    draw_line(
        rect.x + 70.0,
        rect.y + rect.h * 0.62,
        rect.x + rect.w - 70.0,
        rect.y + rect.h * 0.62,
        3.0,
        line(),
    );

    for i in 0..7 {
        let y = rect.y + rect.h * 0.66 + i as f32 * 22.0;
        draw_line(
            rect.x + 80.0,
            y,
            rect.x + rect.w - 80.0,
            y + 18.0,
            1.0,
            line(),
        );
    }

    let doors = [
        (rect.x + 185.0, "STUDIO 9"),
        (rect.x + 405.0, "REDAKTION"),
        (rect.x + 605.0, "KÜCHE"),
        (rect.x + 790.0, "BESPRECHUNG"),
        (rect.x + 1000.0, "SERVER 11"),
    ];
    for (x, label) in doors {
        draw_rectangle(x, rect.y + 158.0, 104.0, 195.0, col(0x23, 0x20, 0x1c));
        draw_rectangle_lines(x, rect.y + 158.0, 104.0, 195.0, 2.0, BLACK);
        draw_rectangle(x + 14.0, rect.y + 188.0, 76.0, 22.0, col(0x10, 0x14, 0x14));
        centered_text(label, x + 52.0, rect.y + 205.0, 12, ochre());
        draw_circle(x + 84.0, rect.y + 264.0, 4.0, ochre());
    }

    draw_rectangle(
        rect.x + 88.0,
        rect.y + 120.0,
        95.0,
        205.0,
        col(0x24, 0x21, 0x1d),
    );
    draw_rectangle_lines(rect.x + 88.0, rect.y + 120.0, 95.0, 205.0, 2.0, BLACK);
    centered_text("HOF", rect.x + 135.0, rect.y + 232.0, 20, muted());

    draw_rectangle(
        rect.x + 270.0,
        rect.y + 108.0,
        130.0,
        90.0,
        col(0x10, 0x15, 0x15),
    );
    draw_rectangle_lines(rect.x + 270.0, rect.y + 108.0, 130.0, 90.0, 2.0, ochre());
    centered_text("DISPO", rect.x + 335.0, rect.y + 140.0, 18, paper());
    centered_text("LIVE / VOD", rect.x + 335.0, rect.y + 166.0, 13, muted());

    draw_rectangle(
        rect.x + 585.0,
        rect.y + 110.0,
        150.0,
        88.0,
        col(0x12, 0x16, 0x17),
    );
    draw_rectangle_lines(rect.x + 585.0, rect.y + 110.0, 150.0, 88.0, 2.0, line());
    centered_text("TECHNIKLAGER", rect.x + 660.0, rect.y + 154.0, 15, paper());
    draw_camera_icon(rect.x + 615.0, rect.y + 120.0, "LEIH");

    for step in 0..6 {
        let x = rect.x + 770.0 + step as f32 * 19.0;
        let y = rect.y + 190.0 - step as f32 * 11.0;
        draw_rectangle(x, y, 48.0, 8.0, line());
    }
    centered_text("TREPPENHAUS", rect.x + 840.0, rect.y + 116.0, 14, muted());

    draw_line(
        rect.x + 930.0,
        rect.y + 405.0,
        rect.x + 1045.0,
        rect.y + 430.0,
        5.0,
        BLACK,
    );
    draw_line(
        rect.x + 932.0,
        rect.y + 394.0,
        rect.x + 1068.0,
        rect.y + 410.0,
        3.0,
        col(0x59, 0x4a, 0x3e),
    );
    centered_text("80 m SDI", rect.x + 1002.0, rect.y + 392.0, 14, ochre());

    centered_text(
        "BÜROFLUR",
        rect.x + rect.w * 0.5,
        rect.y + 78.0,
        30,
        paper(),
    );
    centered_text(
        "Heinrichstraße 9-11, öffentlich rekonstruiert",
        rect.x + rect.w * 0.5,
        rect.y + 110.0,
        16,
        muted(),
    );
    draw_rectangle(
        rect.x + rect.w * 0.80,
        rect.y + 106.0,
        120.0,
        78.0,
        col(0x10, 0x15, 0x15),
    );
    draw_rectangle_lines(
        rect.x + rect.w * 0.80,
        rect.y + 106.0,
        120.0,
        78.0,
        2.0,
        ochre(),
    );
    centered_text(
        "PLAN",
        rect.x + rect.w * 0.80 + 60.0,
        rect.y + 151.0,
        18,
        paper(),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, rust());
}

fn draw_courtyard_room(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col(0x1d, 0x20, 0x1f));
    draw_rectangle(rect.x, rect.y, rect.w, rect.h * 0.55, col(0x36, 0x34, 0x2f));
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.55,
        rect.w,
        rect.h * 0.45,
        col(0x21, 0x22, 0x20),
    );

    let houses = [
        (rect.x + 110.0, 250.0, "HAUS 9", "STUDIO"),
        (rect.x + 450.0, 270.0, "HAUS 11", "BÜROS / SERVER"),
        (rect.x + 825.0, 245.0, "HAUS 15", "SZENENBAU"),
    ];
    for (x, w, title, sub) in houses {
        draw_rectangle(x, rect.y + 72.0, w, 240.0, col(0x2a, 0x24, 0x20));
        draw_rectangle_lines(x, rect.y + 72.0, w, 240.0, 2.0, BLACK);
        for col_idx in 0..3 {
            let wx = x + 30.0 + col_idx as f32 * 70.0;
            draw_rectangle(wx, rect.y + 108.0, 42.0, 42.0, col(0x12, 0x18, 0x1a));
            draw_rectangle(wx, rect.y + 174.0, 42.0, 42.0, col(0x12, 0x18, 0x1a));
        }
        centered_text(title, x + w * 0.5, rect.y + 256.0, 24, paper());
        centered_text(sub, x + w * 0.5, rect.y + 282.0, 14, muted());
    }

    draw_rectangle(
        rect.x + rect.w * 0.15,
        rect.y + rect.h * 0.62,
        210.0,
        52.0,
        col(0x39, 0x32, 0x2d),
    );
    centered_text(
        "LADEZONE",
        rect.x + rect.w * 0.15 + 105.0,
        rect.y + rect.h * 0.62 + 34.0,
        17,
        muted(),
    );
    draw_rectangle(
        rect.x + rect.w * 0.58,
        rect.y + rect.h * 0.27,
        120.0,
        52.0,
        col(0x10, 0x12, 0x12),
    );
    centered_text(
        "9-11",
        rect.x + rect.w * 0.58 + 60.0,
        rect.y + rect.h * 0.27 + 34.0,
        22,
        ochre(),
    );
    centered_text(
        "HEINRICHSTRASSE",
        rect.x + rect.w * 0.5,
        rect.y + 48.0,
        26,
        paper(),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, rust());
}

fn draw_server_room(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col(0x10, 0x13, 0x14));
    draw_rectangle(rect.x, rect.y, rect.w, rect.h * 0.58, col(0x18, 0x20, 0x22));
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.58,
        rect.w,
        rect.h * 0.42,
        col(0x0e, 0x10, 0x11),
    );

    for rack in 0..3 {
        let x = rect.x + 250.0 + rack as f32 * 92.0;
        draw_rectangle(x, rect.y + 118.0, 72.0, 260.0, col(0x12, 0x16, 0x18));
        draw_rectangle_lines(x, rect.y + 118.0, 72.0, 260.0, 2.0, line());
        for slot in 0..9 {
            let y = rect.y + 136.0 + slot as f32 * 24.0;
            draw_rectangle(x + 12.0, y, 48.0, 8.0, col(0x27, 0x30, 0x31));
            draw_circle(x + 56.0, y + 4.0, 2.0, ochre());
        }
    }

    draw_rectangle(
        rect.x + 610.0,
        rect.y + 150.0,
        240.0,
        140.0,
        col(0x0b, 0x12, 0x13),
    );
    draw_rectangle_lines(rect.x + 610.0, rect.y + 150.0, 240.0, 140.0, 2.0, ochre());
    centered_text("VIDEO-HUB", rect.x + 730.0, rect.y + 218.0, 22, paper());
    for i in 0..6 {
        draw_line(
            rect.x + 620.0,
            rect.y + 292.0 + i as f32 * 10.0,
            rect.x + 845.0,
            rect.y + 292.0 + i as f32 * 10.0,
            2.0,
            line(),
        );
    }

    draw_circle_lines(rect.x + 965.0, rect.y + 320.0, 54.0, 8.0, rust());
    draw_circle_lines(rect.x + 965.0, rect.y + 320.0, 31.0, 8.0, line());
    centered_text("80 m SDI", rect.x + 965.0, rect.y + 395.0, 17, ochre());
    draw_rectangle(
        rect.x + 925.0,
        rect.y + 125.0,
        170.0,
        94.0,
        col(0x0b, 0x12, 0x13),
    );
    draw_rectangle_lines(rect.x + 925.0, rect.y + 125.0, 170.0, 94.0, 2.0, line());
    centered_text("BACKUP", rect.x + 1010.0, rect.y + 166.0, 19, muted());
    centered_text("STREAM", rect.x + 1010.0, rect.y + 194.0, 19, muted());

    centered_text(
        "SERVERRAUM 11",
        rect.x + rect.w * 0.5,
        rect.y + 66.0,
        30,
        paper(),
    );
    centered_text(
        "Kreuzschiene, Racks und die lange Spur zum Studio 9",
        rect.x + rect.w * 0.5,
        rect.y + 96.0,
        15,
        muted(),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, rust());
}

fn draw_set_workshop_room(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col(0x1f, 0x1b, 0x17));
    draw_rectangle(rect.x, rect.y, rect.w, rect.h * 0.58, col(0x35, 0x2d, 0x25));
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.58,
        rect.w,
        rect.h * 0.42,
        col(0x18, 0x16, 0x13),
    );

    for i in 0..5 {
        let x = rect.x + 205.0 + i as f32 * 62.0;
        draw_rectangle(
            x,
            rect.y + 145.0 - i as f32 * 7.0,
            44.0,
            205.0,
            col(0x49, 0x38, 0x2e),
        );
        draw_rectangle_lines(x, rect.y + 145.0 - i as f32 * 7.0, 44.0, 205.0, 2.0, BLACK);
    }

    draw_rectangle(
        rect.x + 610.0,
        rect.y + 280.0,
        250.0,
        72.0,
        col(0x45, 0x35, 0x2d),
    );
    draw_rectangle_lines(rect.x + 610.0, rect.y + 280.0, 250.0, 72.0, 2.0, BLACK);
    for i in 0..7 {
        draw_circle(
            rect.x + 642.0 + i as f32 * 28.0,
            rect.y + 303.0 + (i % 2) as f32 * 14.0,
            8.0,
            if i % 2 == 0 { ochre() } else { rust() },
        );
    }
    centered_text("MALTISCH", rect.x + 735.0, rect.y + 340.0, 17, paper());

    draw_rectangle(
        rect.x + 870.0,
        rect.y + 130.0,
        170.0,
        105.0,
        col(0x14, 0x16, 0x15),
    );
    draw_rectangle_lines(rect.x + 870.0, rect.y + 130.0, 170.0, 105.0, 2.0, line());
    centered_text("ALTE", rect.x + 955.0, rect.y + 176.0, 18, muted());
    centered_text("WERKSTATT", rect.x + 955.0, rect.y + 202.0, 18, muted());

    draw_rectangle(
        rect.x + 955.0,
        rect.y + 255.0,
        120.0,
        120.0,
        col(0x22, 0x20, 0x1d),
    );
    draw_rectangle_lines(rect.x + 955.0, rect.y + 255.0, 120.0, 120.0, 2.0, ochre());
    centered_text("STUDIO", rect.x + 1015.0, rect.y + 323.0, 20, paper());

    centered_text(
        "HAUS-15-SZENENBAU",
        rect.x + rect.w * 0.5,
        rect.y + 68.0,
        30,
        paper(),
    );
    centered_text(
        "ehemalige Werkstatt, Kulissen, Tape und Materialwege",
        rect.x + rect.w * 0.5,
        rect.y + 98.0,
        15,
        muted(),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, rust());
}

fn draw_camera_icon(x: f32, y: f32, label: &str) {
    draw_rectangle(x, y, 58.0, 34.0, col(0x12, 0x16, 0x17));
    draw_rectangle(x + 58.0, y + 8.0, 22.0, 16.0, col(0x12, 0x16, 0x17));
    draw_rectangle_lines(x, y, 80.0, 34.0, 2.0, line());
    draw_line(x + 16.0, y + 34.0, x + 4.0, y + 74.0, 3.0, line());
    draw_line(x + 42.0, y + 34.0, x + 55.0, y + 74.0, 3.0, line());
    centered_text(label, x + 29.0, y + 23.0, 13, paper());
}

fn draw_placeholder_sprite(rect: Rect, label: &str, category: PlaceholderCategory, selected: bool) {
    let (bg, fg) = category.colors();
    let fill = if selected { lighten(bg, 0.18) } else { bg };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BLACK);
    draw_rectangle_lines(
        rect.x + 2.0,
        rect.y + 2.0,
        (rect.w - 4.0).max(1.0),
        (rect.h - 4.0).max(1.0),
        1.0,
        Color::new(1.0, 1.0, 1.0, 0.18),
    );
    draw_placeholder_label(rect, label, fg);
}

fn draw_placeholder_label(rect: Rect, label: &str, color: Color) {
    let text = label.replace(' ', "\n");
    let lines: Vec<&str> = text.lines().take(4).collect();
    if lines.is_empty() {
        return;
    }
    let longest = lines.iter().map(|l| l.len()).max().unwrap_or(1) as f32;
    let by_width = ((rect.w - 8.0).max(12.0) / (longest * 0.58)).floor();
    let by_height = ((rect.h - 8.0).max(10.0) / (lines.len() as f32 * 1.18)).floor();
    let size = by_width.min(by_height).clamp(9.0, 22.0);
    let line_h = size * 1.16;
    let total_h = lines.len() as f32 * line_h;
    let mut y = rect.y + (rect.h - total_h) * 0.5 + size;
    for line in lines {
        let dims = measure_text(line, None, size as u16, 1.0);
        let x = rect.x + (rect.w - dims.width) * 0.5;
        draw_outlined_text(line, x, y, size, color);
        y += line_h;
    }
}

fn draw_outlined_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        draw_text(text, x + dx, y + dy, size, BLACK);
    }
    draw_text(text, x, y, size, color);
}

fn draw_inventory_item(
    item: &ItemMeta,
    rect: Rect,
    icons: Option<&Texture2D>,
    selected: bool,
    hovered: bool,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.03, 0.04, 0.04, 0.72),
    );
    if selected {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, ochre());
    } else if hovered {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, amber());
    } else {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, line());
    }

    let icon_rect = Rect::new(rect.x + 4.0, rect.y + 4.0, rect.w - 8.0, rect.h - 8.0);
    if let Some(texture) = icons {
        if let Some(source) = inventory_icon_source(item.id, texture) {
            draw_texture_ex(
                texture,
                icon_rect.x,
                icon_rect.y,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    dest_size: Some(vec2(icon_rect.w, icon_rect.h)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    draw_placeholder_sprite(icon_rect, item.short, PlaceholderCategory::Prop, selected);
}

fn inventory_icon_source(item_id: &str, texture: &Texture2D) -> Option<Rect> {
    let (col, row) = match item_id {
        "call_sheet" => (0.0, 0.0),
        "gaffer_tape" => (1.0, 0.0),
        "sdi_label" => (2.0, 0.0),
        "lower_third_card" => (0.0, 1.0),
        "city_reflector" => (1.0, 1.0),
        "cardboard_piece" => (0.0, 0.0),
        "clear_tape" => (1.0, 0.0),
        "busker_sign" => (0.0, 1.0),
        "mold_token" => (3.0, 1.0),
        "medical_release" => (4.0, 0.0),
        "checksum_note" => (4.0, 1.0),
        _ => return None,
    };
    let cell_w = texture.width() / 5.0;
    let cell_h = texture.height() / 2.0;
    Some(Rect::new(col * cell_w, row * cell_h, cell_w, cell_h))
}

fn draw_footer_notebook(game: &Game) {
    draw_text_ex("FALLAKTE", 936.0, 628.0, text_params(14, ochre()));
    draw_text_ex(
        notebook_line(game, 0),
        936.0,
        649.0,
        text_params(13, paper()),
    );
    draw_text_ex(
        notebook_line(game, 1),
        936.0,
        668.0,
        text_params(13, muted()),
    );
    draw_text_ex(
        notebook_line(game, 2),
        936.0,
        687.0,
        text_params(13, muted()),
    );
}

fn notebook_line(game: &Game, line: usize) -> &'static str {
    if game.state.scene == "hospital_room" {
        return match line {
            0 if !game.flag("hospital_vitals_checked") => "Ansehen: Monitorwerte",
            0 => "Werte stabil",
            1 if !game.flag("nurse_called") => "Benutzen: Rufknopf",
            1 if !game.flag("nurse_clearance") => "Reden: Pflegekraft",
            1 => "Entlassung freigegeben",
            2 if !game.has_item("medical_release") => "Aufheben: Entlassungsbogen",
            _ => "Benutzen: Korridortür",
        };
    }
    if game.state.scene == "sprinkler_courtyard" {
        return match line {
            0 if !game.flag("fire_alarm_logged") => "Benutzen: Alarmfeld",
            0 => "Alarmkreis bestätigt",
            1 if !game.flag("fire_cabinet_checked") => "Benutzen: Löschschrank",
            1 if !game.flag("safety_clearance") => "Reden: Sicherheitsdienst",
            1 => "Sicherheit freigegeben",
            _ => "Benutzen: Rückweg",
        };
    }
    if game.state.scene == "prop_storage_collapse" {
        return match line {
            0 if !game.flag("collapse_braced") => "Benutzen: Stützstrebe",
            0 => "Strebe sitzt",
            1 if !game.flag("collapse_rope_released") => "Benutzen: Notleine",
            1 => "Riegel offen",
            _ => "Benutzen: Werkstatttür",
        };
    }
    if game.state.scene == "archive_recovery" {
        return match line {
            0 if !game.flag("archive_drive_checked") => "Ansehen: Archivlaufwerk",
            0 => "Quelle lesbar",
            1 if !game.has_item("checksum_note") => "Aufheben: Prüfsummenzettel",
            1 if !game.flag("recovery_session_closed") => "Benutzen: Terminal",
            1 => "Session geschlossen",
            _ => "Benutzen: Tür zur Regie",
        };
    }
    if game.state.scene == "video_schimmelbrueder" {
        return match line {
            0 if !game.flag("schimmel_floor_checked") => "Ansehen: Hallenboden",
            0 => "Standflaeche gefunden",
            1 if !game.has_item("mold_token") => "Aufheben: Frame-Probe",
            1 if !game.flag("mold_material_checked") => "Probe mit Formenreihe",
            1 if !game.flag("mold_pattern_checked") => "Probe an Musterbahn",
            1 => "Probe kalibriert",
            2 if !game.flag("schimmel_video_cleared") => "Frame-Probe am Ausgang",
            _ => "Zurueck in die Regie",
        };
    }
    match line {
        0 if !game.has_item("call_sheet") => "Aufheben: Fallakte 000",
        0 if !game.flag("greenscreen_marked") => "Bodenmarken mit Tape setzen",
        0 => "Greenscreen markiert",
        1 if !game.has_item("sdi_label") => "Aufheben: SDI-Label",
        1 if !game.flag("route_labeled") => "Signalweg beschriften",
        1 if !game.has_item("lower_third_card") => "Aufheben: Host-Karte",
        1 if !game.flag("graphic_loaded") => "Host-Karte in Regie laden",
        1 if !game.flag("schimmel_video_cleared") => "Regie: Archivframe sichern",
        1 => "Regie vorbereitet",
        2 if !game.flag("station_timed") => "Zeitreferenz am Bahnhof prüfen",
        2 if !game.flag("city_light_checked") => "Lichtreferenz am Wasserturm prüfen",
        2 if !game.flag("city_notice_checked") => "Drehhinweis an der Schanze lesen",
        2 if !game.has_item("cardboard_piece") => "Pappstück aus Marktkisten holen",
        2 if !game.has_item("clear_tape") => "Transparentband am Kiosk holen",
        2 if !game.has_item("busker_sign") => "Kofferschild im Copyshop drucken",
        2 if !game.has_item("medical_release") => "Entlassungsbogen als Nachweis holen",
        2 if !game.flag("busker_release_shown") => "Entlassung beim Musiker zeigen",
        2 if !game.flag("busker_case_fixed") => "Schild am Koffer befestigen",
        2 if !game.has_item("city_reflector") => "Kofferreflektor vom Musiker holen",
        2 if !game.flag("city_reflector_placed") => "Lichtreferenz im Studio setzen",
        _ => "Benutzen: Rettungslauf starten",
    }
}

fn draw_background() {
    clear_background(paper_black());
    draw_rectangle(0.0, 0.0, VW, VH, paper_black());
}

fn panel(r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, ui_dark());
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, rust());
}

fn overlay() {
    draw_rectangle(0.0, 0.0, VW, VH, Color::new(0.0, 0.0, 0.0, 0.72));
}

fn button(rect: Rect, text: &str, selected: bool, mouse: Vec2) {
    let hover = rect.contains(mouse);
    let fill = if selected {
        rust()
    } else if hover {
        col(0x2f, 0x2a, 0x23)
    } else {
        ui_mid()
    };
    let stroke = if selected { ochre() } else { rust() };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, BLACK);
    draw_rectangle_lines(
        rect.x + 2.0,
        rect.y + 2.0,
        rect.w - 4.0,
        rect.h - 4.0,
        1.0,
        stroke,
    );
    centered_text(
        &text.to_uppercase(),
        rect.x + rect.w / 2.0,
        rect.y + rect.h / 2.0 + 6.0,
        18,
        paper(),
    );
}

fn draw_verb_button(verb: Verb, rect: Rect, selected: bool, icons: Option<&Texture2D>) {
    let fill = if selected { rust() } else { ui_mid() };
    let stroke = if selected { ochre() } else { line() };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BLACK);
    draw_rectangle_lines(
        rect.x + 2.0,
        rect.y + 2.0,
        rect.w - 4.0,
        rect.h - 4.0,
        1.0,
        stroke,
    );

    if let Some(texture) = icons {
        let source = verb_icon_source(verb, texture);
        let icon_rect = Rect::new(rect.x + 6.0, rect.y + 3.0, rect.w - 12.0, rect.h - 6.0);
        draw_texture_ex(
            texture,
            icon_rect.x,
            icon_rect.y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(icon_rect.w, icon_rect.h)),
                ..Default::default()
            },
        );
    } else {
        let color = if selected { paper() } else { muted() };
        draw_verb_icon(verb, rect, color);
    }
}

fn verb_icon_source(verb: Verb, texture: &Texture2D) -> Rect {
    let index = Verb::ALL
        .iter()
        .position(|candidate| *candidate == verb)
        .unwrap_or(0);
    let col = (index % 4) as f32;
    let row = (index / 4) as f32;
    let cell_w = texture.width() / 4.0;
    let cell_h = texture.height() / 2.0;
    Rect::new(col * cell_w, row * cell_h, cell_w, cell_h)
}

fn draw_verb_icon(verb: Verb, rect: Rect, color: Color) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    match verb {
        Verb::Look => {
            draw_line(cx - 15.0, cy, cx - 6.0, cy - 6.0, 2.0, color);
            draw_line(cx - 6.0, cy - 6.0, cx + 6.0, cy - 6.0, 2.0, color);
            draw_line(cx + 6.0, cy - 6.0, cx + 15.0, cy, 2.0, color);
            draw_line(cx - 15.0, cy, cx - 6.0, cy + 6.0, 2.0, color);
            draw_line(cx - 6.0, cy + 6.0, cx + 6.0, cy + 6.0, 2.0, color);
            draw_line(cx + 6.0, cy + 6.0, cx + 15.0, cy, 2.0, color);
            draw_circle(cx, cy, 3.2, color);
        }
        Verb::Poke => {
            draw_line(cx - 14.0, cy + 3.0, cx + 10.0, cy - 4.0, 3.0, color);
            draw_circle(cx + 13.0, cy - 5.0, 3.0, color);
            draw_circle_lines(cx + 18.0, cy - 6.0, 5.0, 1.5, color);
        }
        Verb::Use => {
            draw_line(cx - 10.0, cy + 7.0, cx + 9.0, cy - 8.0, 3.0, color);
            draw_circle_lines(cx - 12.0, cy + 8.0, 5.0, 2.0, color);
            draw_line(cx + 5.0, cy - 8.0, cx + 14.0, cy - 8.0, 2.0, color);
            draw_line(cx + 9.0, cy - 11.0, cx + 9.0, cy - 4.0, 2.0, color);
        }
        Verb::PickUp => {
            draw_line(cx - 12.0, cy - 8.0, cx + 3.0, cy + 3.0, 3.0, color);
            draw_line(cx + 3.0, cy + 3.0, cx + 12.0, cy - 3.0, 2.5, color);
            draw_circle_lines(cx + 11.0, cy + 7.0, 5.0, 2.0, color);
            draw_line(cx - 7.0, cy + 10.0, cx + 16.0, cy + 10.0, 1.5, color);
        }
        Verb::Tongue => {
            draw_circle_lines(cx - 7.0, cy - 3.0, 6.0, 2.0, color);
            draw_line(cx - 3.0, cy + 2.0, cx + 8.0, cy + 6.0, 3.0, color);
            draw_circle(cx + 12.0, cy + 7.0, 3.0, color);
            draw_line(cx + 15.0, cy - 6.0, cx + 15.0, cy + 9.0, 2.0, color);
        }
        Verb::Smell => {
            draw_line(cx - 9.0, cy - 7.0, cx - 2.0, cy, 2.0, color);
            draw_line(cx - 2.0, cy, cx - 8.0, cy + 7.0, 2.0, color);
            draw_line(cx - 8.0, cy + 7.0, cx + 1.0, cy + 7.0, 2.0, color);
            for i in 0..3 {
                let x = cx + 7.0 + i as f32 * 5.0;
                draw_line(x, cy + 5.0, x + 3.0, cy - 5.0, 1.4, color);
            }
        }
        Verb::Talk => {
            draw_rectangle_lines(cx - 13.0, cy - 8.0, 26.0, 14.0, 2.0, color);
            draw_triangle(
                vec2(cx - 4.0, cy + 6.0),
                vec2(cx + 2.0, cy + 6.0),
                vec2(cx - 6.0, cy + 12.0),
                color,
            );
            draw_circle(cx - 6.0, cy - 1.0, 1.7, color);
            draw_circle(cx, cy - 1.0, 1.7, color);
            draw_circle(cx + 6.0, cy - 1.0, 1.7, color);
        }
    }
}

fn draw_dialogue_text_box(rect: Rect, text: &str, color: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.03, 0.05, 0.06, 0.70),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, line());
    draw_text_wrapped(
        text,
        rect.x + 16.0,
        rect.y + 25.0,
        rect.w - 32.0,
        18.0,
        color,
    );
}

fn draw_dialogue_choice(rect: Rect, text: &str, mouse: Vec2) {
    let hover = rect.contains(mouse);
    let fill = if hover {
        col(0x24, 0x31, 0x35)
    } else {
        col(0x10, 0x17, 0x19)
    };
    let stroke = if hover { amber() } else { line() };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BLACK);
    draw_rectangle_lines(
        rect.x + 2.0,
        rect.y + 2.0,
        rect.w - 4.0,
        rect.h - 4.0,
        1.0,
        stroke,
    );
    draw_text_ex(">", rect.x + 16.0, rect.y + 28.0, text_params(18, amber()));
    draw_text_ex(text, rect.x + 38.0, rect.y + 28.0, text_params(17, paper()));
}

fn draw_hover(text: &str, mouse: Vec2) {
    let size = 15;
    let max_text_w = 500.0;
    let hover_text = text.to_uppercase();
    let lines = wrap_lines(&hover_text, max_text_w, size);
    let line_h = 18.0;
    let text_w = lines
        .iter()
        .map(|line| measure_text(line, None, size, 1.0).width)
        .fold(0.0, f32::max);
    let w = text_w + 24.0;
    let h = lines.len() as f32 * line_h + 18.0;
    let mut x = mouse.x + 16.0;
    let mut y = mouse.y + 18.0;
    if x + w > VW - 16.0 {
        x = VW - w - 16.0;
    }
    if y + h > VH - 16.0 {
        y = mouse.y - h - 16.0;
    }
    x = x.max(16.0);
    y = y.max(16.0);

    draw_rectangle(x, y, w, h, Color::new(0.09, 0.07, 0.06, 0.94));
    draw_rectangle_lines(x, y, w, h, 2.0, ochre());
    for (index, line) in lines.iter().enumerate() {
        draw_text_ex(
            line,
            x + 12.0,
            y + 22.0 + index as f32 * line_h,
            text_params(size, paper()),
        );
    }
}

fn wrap_lines(text: &str, max_w: f32, size: u16) -> Vec<String> {
    let mut lines = Vec::new();
    for segment in text.lines() {
        let mut line = String::new();
        for word in segment.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };
            if measure_text(&candidate, None, size, 1.0).width > max_w && !line.is_empty() {
                lines.push(line);
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn centered_text(text: &str, x: f32, y: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text_ex(text, x - dims.width / 2.0, y, text_params(size, color));
}

fn draw_text_wrapped(text: &str, x: f32, y: f32, max_w: f32, line_h: f32, color: Color) -> f32 {
    let size = line_h as u16;
    let mut line = String::new();
    let mut cy = y;
    let mut height = 0.0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if measure_text(&candidate, None, size, 1.0).width > max_w && !line.is_empty() {
            draw_text_ex(&line, x, cy, text_params(size, color));
            cy += line_h + 4.0;
            height += line_h + 4.0;
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_text_ex(&line, x, cy, text_params(size, color));
        height += line_h + 4.0;
    }
    height
}

fn text_params(size: u16, color: Color) -> TextParams<'static> {
    TextParams {
        font_size: size,
        color,
        ..Default::default()
    }
}

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn lighten(color: Color, amount: f32) -> Color {
    Color::new(
        (color.r + amount).min(1.0),
        (color.g + amount).min(1.0),
        (color.b + amount).min(1.0),
        color.a,
    )
}

fn paper_black() -> Color {
    col(0x1a, 0x14, 0x10)
}

fn ui_dark() -> Color {
    col(0x17, 0x12, 0x10)
}

fn ui_mid() -> Color {
    col(0x2a, 0x22, 0x1c)
}

fn rust() -> Color {
    col(0xb8, 0x5c, 0x3c)
}

fn paper() -> Color {
    col(0xe8, 0xdc, 0xc4)
}

fn ochre() -> Color {
    col(0xd4, 0x9b, 0x3f)
}

fn line() -> Color {
    col(0x59, 0x4a, 0x3e)
}

fn bone() -> Color {
    paper()
}

fn muted() -> Color {
    col(0xa8, 0x9a, 0x82)
}

fn amber() -> Color {
    ochre()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lost_klixx_loop_can_complete() {
        clear_save();
        let mut game = Game::new(GameState::default());

        assert_eq!(game.state.scene, "greenscreen_studio");

        game.travel("office_hall", "test");
        game.pick_up_hotspot("dispo_board");
        assert!(game.has_item("call_sheet"));

        game.travel("building_courtyard", "test");
        game.travel("set_workshop", "test");
        game.pick_up_hotspot("gaffer_roll");
        assert!(game.has_item("gaffer_tape"));

        game.travel("server_room", "test");
        game.inspect("video_hub");
        assert!(game.flag("route_checked"));
        game.pick_up_hotspot("sdi_label_printer");
        assert!(game.has_item("sdi_label"));

        game.travel("schanzenstrasse", "test");
        game.travel("sternschanze_station", "test");
        game.inspect("station_clock");
        assert!(game.flag("station_timed"));
        game.travel("schanzenpark", "test");
        game.inspect("water_tower");
        assert!(game.flag("city_light_checked"));

        game.travel("karoviertel", "test");
        game.pick_up_hotspot("print_shop");
        assert!(game.has_item("lower_third_card"));
        game.pick_up_hotspot("market_boxes");
        assert!(game.has_item("cardboard_piece"));

        game.travel("schanzenstrasse", "test");
        game.inspect("city_notice");
        assert!(game.flag("city_notice_checked"));
        game.pick_up_hotspot("corner_kiosk");
        assert!(game.has_item("clear_tape"));
        game.travel("karoviertel", "test");
        assert!(game
            .hotspots()
            .iter()
            .any(|hotspot| hotspot.id == "print_shop"));
        game.handle_item_use("print_shop", "cardboard_piece");
        assert!(game.has_item("busker_sign"));

        game.add_item("medical_release");
        game.travel("sternschanze_station", "test");
        game.handle_item_use("busker_case", "medical_release");
        assert!(game.flag("busker_release_shown"));
        game.handle_item_use("busker_case", "busker_sign");
        assert!(game.flag("busker_sign_positioned"));
        game.handle_item_use("busker_case", "clear_tape");
        assert!(game.flag("busker_case_fixed"));
        assert!(game.has_item("city_reflector"));

        game.travel("control_room", "test");
        game.handle_item_use("graphics_terminal", "lower_third_card");
        assert!(game.flag("graphic_loaded"));
        game.use_hotspot("video_schimmelbrueder");
        game.inspect("factory_floor");
        game.pick_up_hotspot("mold_token");
        game.handle_item_use("mold_rack", "mold_token");
        assert!(game.flag("mold_material_checked"));
        game.handle_item_use("patterned_belt", "mold_token");
        assert!(game.flag("mold_pattern_checked"));
        game.handle_item_use("song_exit_gate", "mold_token");
        assert!(game.flag("schimmel_video_cleared"));
        game.use_hotspot("song_exit_gate");
        assert_eq!(game.state.scene, "control_room");

        game.travel("greenscreen_studio", "test");
        game.handle_item_use("floor_marks", "gaffer_tape");
        assert!(game.flag("greenscreen_marked"));
        game.handle_item_use("greenscreen_setup", "sdi_label");
        assert!(game.flag("route_labeled"));
        game.handle_item_use("greenscreen_setup", "city_reflector");
        assert!(game.flag("city_reflector_placed"));
        game.use_hotspot("greenscreen_setup");

        assert!(game.state.complete);
        assert!(game.flag("rescue_ready"));
        assert!(matches!(game.modal, Modal::Milestone));

        clear_save();
    }

    #[test]
    fn rescue_requires_city_checks_frame_and_setup_items() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.use_hotspot("greenscreen_setup");
        assert!(matches!(game.modal, Modal::None));
        assert!(game.status.contains("Fallakte"));

        game.add_item("call_sheet");
        game.state.flags.insert("greenscreen_marked".to_string());
        game.state.flags.insert("route_labeled".to_string());
        game.state.flags.insert("graphic_loaded".to_string());
        game.state.flags.insert("station_timed".to_string());
        game.state.flags.insert("city_light_checked".to_string());

        game.use_hotspot("greenscreen_setup");
        assert!(!game.state.complete);
        assert!(game.status.contains("Schimmelbrüder"));

        game.state
            .flags
            .insert("schimmel_video_cleared".to_string());
        game.use_hotspot("greenscreen_setup");
        assert!(!game.state.complete);
        assert!(game.status.contains("Lichtreferenz"));

        game.state.flags.insert("city_reflector_placed".to_string());
        game.use_hotspot("greenscreen_setup");
        assert!(game.state.complete);

        clear_save();
    }

    #[test]
    fn dialogue_choice_stays_inside_dialogue_modal() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.talk("mentor_shadow");
        assert!(matches!(game.modal, Modal::Dialogue("mentor")));
        game.update_dialogue("mentor", macroquad::prelude::vec2(200.0, 320.0));

        let response = game.dialogue_response.as_ref().expect("dialogue response");
        assert_eq!(response.dialogue_id, "mentor");
        assert!(response.text.contains("Fallakte"));
        assert!(!game.status.contains("Fallakte"));

        clear_save();
    }

    #[test]
    fn office_building_navigation_is_registered() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("office_hall", "test");
        assert_eq!(game.state.scene, "office_hall");
        assert_eq!(current_scene(&game.state.scene).name, "Büroflur");
        assert!(game.hotspot("server_room").is_some());
        assert!(game.hotspot("dispo_board").is_some());

        game.travel("building_courtyard", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Heinrichstraße-Hof");
        assert!(game.hotspot("set_workshop").is_some());

        game.travel("server_room", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Serverraum 11");
        assert!(game.hotspot("sdi_spool").is_some());

        game.travel("set_workshop", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Haus-15-Szenenbau");
        assert!(game.hotspot("bike_workshop_trace").is_some());

        game.travel("schanzenstrasse", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Schanzenstraße");
        assert!(game.hotspot("sternschanze_station").is_some());

        game.travel("sternschanze_station", "test");
        assert_eq!(
            current_scene(&game.state.scene).name,
            "Bahnhof Sternschanze"
        );
        assert!(game.hotspot("schanzenpark").is_some());

        game.travel("schanzenpark", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Schanzenpark");

        game.travel("karoviertel", "test");
        assert_eq!(current_scene(&game.state.scene).name, "Karoviertel");

        clear_save();
    }

    #[test]
    fn exits_work_with_any_verb_and_selected_item() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("office_hall", "test");
        game.state.verb = Verb::Tongue;
        game.add_item("gaffer_tape");
        game.state.selected_item = Some("gaffer_tape".to_string());
        game.handle_hotspot("greenscreen_studio");

        assert_eq!(game.state.scene, "greenscreen_studio");
        assert!(game.state.selected_item.is_none());

        clear_save();
    }

    #[test]
    fn pickups_require_aufheben() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("office_hall", "test");
        game.state.verb = Verb::Use;
        game.handle_hotspot("dispo_board");
        assert!(!game.has_item("call_sheet"));
        assert!(game.status.contains("Aufheben"));

        game.state.verb = Verb::PickUp;
        game.handle_hotspot("dispo_board");
        assert!(game.has_item("call_sheet"));
        assert!(game.status.contains("aufgehoben"));

        clear_save();
    }

    #[test]
    fn case_file_and_frame_probe_have_stateful_object_uses() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.add_item("call_sheet");
        game.travel("sternschanze_station", "test");
        game.handle_item_use("station_clock", "call_sheet");
        assert!(game.flag("station_timed"));
        assert!(game.status.contains("Zeitmarke"));

        game.travel("schanzenpark", "test");
        game.handle_item_use("water_tower", "call_sheet");
        assert!(game.flag("city_light_checked"));

        game.travel("video_schimmelbrueder", "test");
        game.add_item("mold_token");
        game.handle_item_use("mold_rack", "mold_token");
        assert!(game.flag("mold_material_checked"));
        game.handle_item_use("patterned_belt", "mold_token");
        assert!(game.flag("mold_pattern_checked"));

        clear_save();
    }

    #[test]
    fn dangerous_verbs_trigger_consequence_routes() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("server_room", "test");
        game.poke("video_hub");
        assert!(matches!(
            game.death.as_ref().map(|death| death.kind),
            Some(DeathKind::Shock)
        ));

        game.reset();
        game.tongue("greenscreen_wall");
        assert!(matches!(
            game.death.as_ref().map(|death| death.kind),
            Some(DeathKind::Fire)
        ));

        game.reset();
        game.travel("control_room", "test");
        game.tongue("rehearsal_monitor");
        assert!(matches!(
            game.death.as_ref().map(|death| death.kind),
            Some(DeathKind::Signal)
        ));

        clear_save();
    }

    #[test]
    fn rack_shock_leads_to_hospital_ministory() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("server_room", "test");
        game.poke("server_racks");
        assert!(matches!(
            game.death.as_ref().map(|death| death.kind),
            Some(DeathKind::Shock)
        ));

        game.complete_consequence_transition();
        assert_eq!(game.state.scene, "hospital_room");

        game.use_hotspot("hospital_exit");
        assert_eq!(game.state.scene, "hospital_room");
        assert!(game.status.contains("Entlassungsbogen"));

        game.look("hospital_monitor");
        assert!(game.flag("hospital_vitals_checked"));
        game.poke("call_button");
        assert!(game.flag("nurse_called"));
        game.state
            .flags
            .insert("nurse_clearance_requested".to_string());
        game.state.flags.insert("nurse_clearance".to_string());
        game.pick_up_hotspot("discharge_clipboard");
        assert!(game.has_item("medical_release"));

        game.state.verb = Verb::Smell;
        game.handle_hotspot("hospital_exit");
        assert_eq!(game.state.scene, "office_hall");

        clear_save();
    }

    #[test]
    fn hospital_and_city_have_specific_comedy_interactions() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("hospital_room", "test");
        game.state.verb = Verb::Tongue;
        game.handle_hotspot("nurse_station");
        assert!(game.status.contains("Nein"));

        game.state.verb = Verb::Poke;
        game.handle_hotspot("call_button");
        assert!(game.flag("nurse_called"));
        let first_status = game.status.clone();
        game.handle_hotspot("call_button");
        assert_ne!(game.status, first_status);
        assert!(game.status.contains("schon raus"));

        game.travel("sternschanze_station", "test");
        game.state.selected_item = Some("mold_token".to_string());
        game.handle_hotspot("busker_case");
        assert!(game.status.contains("Frame-Probe"));

        clear_save();
    }

    #[test]
    fn busker_reflector_chain_uses_release_and_city_items() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("schanzenpark", "test");
        game.pick_up_hotspot("city_reflector");
        assert!(!game.has_item("city_reflector"));
        assert!(game.status.contains("Straßenmusikers"));

        game.travel("karoviertel", "test");
        game.handle_item_use("print_shop", "cardboard_piece");
        assert!(!game.has_item("busker_sign"));
        assert!(game.status.contains("Drehhinweis"));

        game.add_item("cardboard_piece");
        game.travel("schanzenstrasse", "test");
        game.inspect("city_notice");
        game.travel("karoviertel", "test");
        assert!(game
            .hotspots()
            .iter()
            .any(|hotspot| hotspot.id == "print_shop"));
        game.handle_item_use("print_shop", "cardboard_piece");
        assert!(game.has_item("busker_sign"));

        game.travel("sternschanze_station", "test");
        game.add_item("clear_tape");
        game.handle_item_use("busker_case", "clear_tape");
        assert!(!game.flag("busker_case_fixed"));
        assert!(game.status.contains("Entlassungsbogen"));

        game.add_item("medical_release");
        game.handle_item_use("busker_case", "medical_release");
        game.handle_item_use("busker_case", "clear_tape");
        assert!(!game.flag("busker_case_fixed"));
        assert!(game.status.contains("Transparentband"));
        game.handle_item_use("busker_case", "busker_sign");
        game.handle_item_use("busker_case", "clear_tape");
        assert!(game.flag("busker_case_fixed"));
        assert!(game.has_item("city_reflector"));

        clear_save();
    }

    #[test]
    fn other_consequence_routes_return_through_small_puzzles() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.trigger_consequence(DeathKind::Fire, "test");
        game.complete_consequence_transition();
        assert_eq!(game.state.scene, "sprinkler_courtyard");
        game.use_hotspot("fire_return_door");
        assert_eq!(game.state.scene, "sprinkler_courtyard");
        game.use_hotspot("alarm_panel");
        game.use_hotspot("extinguisher_cabinet");
        game.state.flags.insert("safety_clearance".to_string());
        game.use_hotspot("fire_return_door");
        assert_eq!(game.state.scene, "building_courtyard");

        game.trigger_consequence(DeathKind::Fall, "test");
        game.complete_consequence_transition();
        assert_eq!(game.state.scene, "prop_storage_collapse");
        game.use_hotspot("brace_beam");
        game.use_hotspot("release_rope");
        game.use_hotspot("collapse_exit");
        assert_eq!(game.state.scene, "set_workshop");

        game.trigger_consequence(DeathKind::Signal, "test");
        game.complete_consequence_transition();
        assert_eq!(game.state.scene, "archive_recovery");
        game.look("archive_drive");
        game.pick_up_hotspot("checksum_printout");
        game.use_hotspot("recovery_terminal");
        game.use_hotspot("control_room_return");
        assert_eq!(game.state.scene, "control_room");

        clear_save();
    }

    #[test]
    fn schimmelbrueder_video_room_can_be_solved() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("control_room", "test");
        game.use_hotspot("video_schimmelbrueder");
        assert_eq!(game.state.scene, "video_schimmelbrueder");
        assert!(game.flag("video_entry_started"));
        assert!(game.status.contains("Monitorbild"));
        assert!(game.hotspot("mold_token").is_some());

        game.use_hotspot("song_exit_gate");
        assert!(!game.flag("schimmel_video_cleared"));
        assert!(game.status.contains("Standfläche"));

        game.inspect("factory_floor");
        assert!(game.flag("schimmel_floor_checked"));

        game.pick_up_hotspot("mold_token");
        assert!(game.has_item("mold_token"));
        assert!(game
            .hotspots()
            .iter()
            .all(|hotspot| hotspot.id != "mold_token"));

        game.handle_item_use("song_exit_gate", "mold_token");
        assert!(!game.flag("schimmel_video_cleared"));
        assert!(game.status.contains("Gleiche"));

        game.smell("mold_rack");
        assert!(game.flag("mold_material_checked"));
        game.inspect("patterned_belt");
        assert!(game.flag("mold_pattern_checked"));

        game.handle_item_use("song_exit_gate", "mold_token");
        assert!(game.flag("schimmel_video_cleared"));
        assert!(game.status.contains("Clip-Ausgang"));
        game.use_hotspot("song_exit_gate");
        assert_eq!(game.state.scene, "control_room");

        clear_save();
    }

    #[test]
    fn hotspots_use_rectangles_until_polygon_is_saved() {
        let scene = current_scene("video_schimmelbrueder");
        let hotspot = HotspotSpec {
            id: "unsaved_test_hotspot",
            name: "Ungespeicherter Test-Hotspot",
            pct: pct(6.18, 12.56, 12.9, 40.64),
            kind: HotspotKind::Prop,
            look: "",
            inspect: "",
            talk_id: None,
        };
        let rect = scene_rect();
        let bounds = pct_rect(rect, hotspot.pct);
        let inside = vec2(bounds.x + bounds.w * 0.5, bounds.y + bounds.h * 0.5);

        assert!(hotspot_contains(scene, &hotspot, rect, inside));
        assert_eq!(hotspot_bounds(scene, &hotspot, rect), bounds);
    }

    #[test]
    fn editor_added_hotspots_are_registered_with_polygons() {
        for (scene_id, hotspot_id) in [
            ("control_room", "mixing_console"),
            ("sternschanze_station", "public_phone"),
            ("sprinkler_courtyard", "cable_bin"),
            ("prop_storage_collapse", "paint_cans"),
        ] {
            let scene = current_scene(scene_id);
            let hotspot = scene
                .hotspots
                .iter()
                .find(|hotspot| hotspot.id == hotspot_id)
                .expect("editor-added hotspot is registered");

            assert!(
                hotspot_polygon(scene, hotspot).is_some(),
                "{scene_id}/{hotspot_id} should use the editor polygon"
            );
        }
    }

    #[test]
    fn selected_video_frames_are_registered_as_scenes() {
        let control_room = current_scene("control_room");

        for scene_id in ["video_icemachine", "video_brassband"] {
            let scene = SCENES
                .iter()
                .find(|scene| scene.id == scene_id)
                .expect("selected frame scene should be registered");
            let asset_path = format!("assets/scenes/{scene_id}.png");

            assert!(
                std::path::Path::new(&asset_path).exists(),
                "{asset_path} should exist for texture loading"
            );
            assert_eq!(
                png_dimensions(&asset_path),
                Some((1240, 480)),
                "{asset_path} should be generated room art at scene resolution, not a raw picked frame"
            );
            assert_eq!(
                file_fingerprint(&asset_path),
                Some(expected_generated_room_asset_fingerprint(scene_id)),
                "{asset_path} changed; keep this as generated room art and update the fingerprint after visual review"
            );
            assert!(
                control_room
                    .hotspots
                    .iter()
                    .any(|hotspot| hotspot.id == scene_id && hotspot.kind == HotspotKind::Exit),
                "control room should link to {scene_id}"
            );
            assert!(
                scene.hotspots.iter().any(
                    |hotspot| hotspot.id == "control_room" && hotspot.kind == HotspotKind::Exit
                ),
                "{scene_id} should link back to the control room"
            );

            for hotspot in scene.hotspots {
                assert!(
                    hotspot_polygon(scene, hotspot).is_some(),
                    "{scene_id}/{} should use an editor polygon",
                    hotspot.id
                );
            }
        }

        let mut game = Game::new(GameState::default());
        game.travel("video_brassband", "test");
        assert!(
            game.hotspots()
                .iter()
                .any(|hotspot| hotspot.id == "brass_players"
                    && hotspot.kind == HotspotKind::Character)
        );
        game.talk("brass_players");
        assert!(game.status.contains("Bläsergruppe"));
        assert!(!game.status.contains("keine Gesprächsebene"));
    }

    fn png_dimensions(path: &str) -> Option<(u32, u32)> {
        let bytes = std::fs::read(path).ok()?;
        if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
            return None;
        }
        Some((
            u32::from_be_bytes(bytes[16..20].try_into().ok()?),
            u32::from_be_bytes(bytes[20..24].try_into().ok()?),
        ))
    }

    fn file_fingerprint(path: &str) -> Option<u64> {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in std::fs::read(path).ok()? {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Some(hash)
    }

    fn expected_generated_room_asset_fingerprint(scene_id: &str) -> u64 {
        match scene_id {
            "video_icemachine" => 0xe896d354e47d79ce,
            "video_brassband" => 0x4d495f1fdee4efd8,
            _ => panic!("unexpected generated video scene id: {scene_id}"),
        }
    }

    #[test]
    fn overlapping_hotspots_prefer_specific_pickups() {
        let scene = current_scene("video_schimmelbrueder");
        let rect = scene_rect();
        let hotspots: Vec<&'static HotspotSpec> = scene.hotspots.iter().collect();
        let token = scene
            .hotspots
            .iter()
            .find(|hotspot| hotspot.id == "mold_token")
            .expect("mold token hotspot");
        let floor = scene
            .hotspots
            .iter()
            .find(|hotspot| hotspot.id == "factory_floor")
            .expect("factory floor hotspot");
        let token_bounds = pct_rect(rect, token.pct);
        let token_center = vec2(
            token_bounds.x + token_bounds.w * 0.5,
            token_bounds.y + token_bounds.h * 0.5,
        );

        assert!(hotspot_contains(scene, token, rect, token_center));
        assert!(hotspot_contains(scene, floor, rect, token_center));
        assert_eq!(
            hovered_hotspot(scene, &hotspots, rect, token_center).map(|hotspot| hotspot.id),
            Some("mold_token")
        );
    }

    #[test]
    fn player_gets_smaller_toward_top_of_walkable_polygon() {
        let scene = current_scene("video_schimmelbrueder");
        let rect = scene_rect();
        let top = pct_point(rect, (56.56, 20.45));
        let bottom = pct_point(rect, (0.0, 100.0));

        assert!(player_depth_scale(top, scene, rect) < player_depth_scale(bottom, scene, rect));
        assert_eq!(
            player_depth_scale(bottom, scene, rect),
            PLAYER_DEPTH_MAX_SCALE
        );
    }

    #[test]
    fn scene_texture_variants_follow_world_state() {
        let mut game = Game::new(GameState::default());
        game.travel("hospital_room", "test");

        assert_eq!(game.scene_texture_key("hospital_room"), "hospital_room");
        assert!(game
            .hotspots()
            .iter()
            .all(|hotspot| hotspot.id != "nurse_station"));
        game.state.flags.insert("nurse_called".to_string());
        assert_eq!(
            game.scene_texture_key("hospital_room"),
            "hospital_room_nurse"
        );
        assert!(game
            .hotspots()
            .iter()
            .any(|hotspot| hotspot.id == "nurse_station"));
        game.add_item("medical_release");
        assert_eq!(
            game.scene_texture_key("hospital_room"),
            "hospital_room_no_clipboard"
        );

        game.add_item("checksum_note");
        assert_eq!(
            game.scene_texture_key("archive_recovery"),
            "archive_recovery_no_checksum"
        );
        game.add_item("mold_token");
        assert_eq!(
            game.scene_texture_key("video_schimmelbrueder"),
            "video_schimmelbrueder_no_token"
        );
    }

    #[test]
    fn walk_cycle_skips_idle_pose() {
        let sampled: Vec<f32> = (0..8)
            .map(|index| walk_cycle_frame(index as f64 / 8.0))
            .collect();

        assert_eq!(sampled, vec![1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0, 2.0]);
    }
}
