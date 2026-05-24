use std::collections::{HashMap, HashSet};

use macroquad::miniquad::conf::{Platform, WebGLVersion};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

const SAVE_KEY: &str = "klixx_rust_save_v10_consequence_routes";
const VW: f32 = 1280.0;
const VH: f32 = 720.0;
const INVENTORY_X: f32 = 540.0;
const INVENTORY_Y: f32 = 620.0;
const INVENTORY_SLOT: f32 = 34.0;
const INVENTORY_GAP: f32 = 5.0;
const INVENTORY_COLUMNS: usize = 9;
const PLAYER_DRAW_W: f32 = 128.0;
const PLAYER_DRAW_H: f32 = 160.0;
const PLAYER_FOOT_ANCHOR_Y: f32 = 0.96;

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
                "Ein Host ist nach dem Öffnen eines Videoeintrags nicht mehr erreichbar. Die Regie dokumentiert den letzten sichtbaren Frame und prüft begehbare Bereiche."
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
        look: "Die Tür zum Büroflur steht offen.",
        inspect: "Im Flur hängen Tagesplan, Raumbelegung und Materiallisten.",
        talk_id: None,
    },
    HotspotSpec {
        id: "control_room",
        name: "Regie",
        pct: pct(94.43, 26.61, 4.85, 51.83),
        kind: HotspotKind::Exit,
        look: "Hinter der Scheibe liegt die Regie mit Monitorwand und Grafikplatz.",
        inspect: "Dort laufen Bild, Ton und Grafik zusammen. Die Signalroute muss eindeutig beschriftet sein.",
        talk_id: None,
    },
    HotspotSpec {
        id: "greenscreen_wall",
        name: "Greenscreen-Fläche",
        pct: pct(38.65, 16.75, 32.76, 47.22),
        kind: HotspotKind::Prop,
        look: "Die grüne Fläche füllt den hinteren Teil des Studios.",
        inspect: "Der Stoff ist sauber gespannt. Für die Probe fehlen neue Bodenmarken.",
        talk_id: None,
    },
    HotspotSpec {
        id: "floor_marks",
        name: "Bodenmarken",
        pct: pct(38.87, 63.25, 22.0, 9.0),
        kind: HotspotKind::Prop,
        look: "Alte Tape-Reste markieren Positionen, die nicht mehr zu diesem Aufbau passen.",
        inspect: "Für die Probe braucht das Studio klare neue Standpunkte.",
        talk_id: None,
    },
    HotspotSpec {
        id: "klixx_table",
        name: "Klixx-Tisch",
        pct: pct(23.54, 51.11, 16.68, 24.39),
        kind: HotspotKind::Prop,
        look: "Ein schmaler Tisch steht vor der Greenscreen-Fläche.",
        inspect: "Die Oberfläche ist frei. Die spätere Kameraposition hängt von den Bodenmarken ab.",
        talk_id: None,
    },
    HotspotSpec {
        id: "chat_preview",
        name: "Chat-Vorschau",
        pct: pct(19.94, 29.56, 4.57, 18.36),
        kind: HotspotKind::Prop,
        look: "Ein Testmonitor zeigt Platzhalter für Chat und Einblendungen.",
        inspect: "Die Vorschau enthält noch keine Bauchbinde. Am Grafikplatz fehlt die freigegebene Karte.",
        talk_id: None,
    },
    HotspotSpec {
        id: "camera_one",
        name: "Studiokamera",
        pct: pct(62.34, 41.44, 10.75, 34.69),
        kind: HotspotKind::Prop,
        look: "Die Kamera ist auf Tisch und Greenscreen eingerichtet.",
        inspect: "Der Bildausschnitt ist eng. Licht und Standpositionen müssen vor dem Probelauf geprüft werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mentor_shadow",
        name: "Aufnahmeleitung",
        pct: pct(75.74, 37.92, 4.67, 34.25),
        kind: HotspotKind::Character,
        look: "Die Aufnahmeleitung steht mit Headset und Klemmbrett am Rand des Studios.",
        inspect: "Sie koordiniert Raumbelegung, Zeitplan und Rückmeldungen aus der Regie.",
        talk_id: Some("mentor"),
    },
    HotspotSpec {
        id: "greenscreen_setup",
        name: "Probelauf-Setup",
        pct: pct(49.18, 35.89, 23.0, 37.0),
        kind: HotspotKind::Prop,
        look: "Hier werden Kamera, Greenscreen und Regiesignal für den Probelauf zusammengeführt.",
        inspect: "Der Probelauf braucht Plan, Bodenmarken, Signalweg, Grafik, Timing und eine Lichtreferenz.",
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
        inspect: "Das On-Air-Licht ist aus. Der Raum wartet auf den Probelauf.",
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
        inspect: "Der Laufzettel nennt ein Außenmotiv im Schanzenviertel.",
        talk_id: None,
    },
    HotspotSpec {
        id: "dispo_board",
        name: "Dispo-Board",
        pct: pct(24.53, 31.94, 8.02, 22.67),
        kind: HotspotKind::Pickup,
        look: "Farbcodes, Zeiten und Raumnummern füllen das Dispo-Board.",
        inspect: "Dein Name steht neben Greenscreen-Probe, Regiecheck und Außenmotiv.",
        talk_id: None,
    },
    HotspotSpec {
        id: "equipment_storage",
        name: "Techniklager",
        pct: pct(43.54, 26.25, 13.0, 40.83),
        kind: HotspotKind::Prop,
        look: "Kamerataschen, Kabel, Mikrofone und Kleinteile stehen griffbereit.",
        inspect: "Die Fächer sind beschriftet. Für die aktuelle Aufgabe wird hier kein weiteres Material benötigt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "staircase",
        name: "Treppe zum Schlauchbüro",
        pct: pct(60.8, 13.11, 8.98, 50.06),
        kind: HotspotKind::Prop,
        look: "Eine Treppe hinauf Richtung Morning Call und langes Büro.",
        inspect: "Die Treppe führt in die Büros. Für den Probelauf ist der Weg nicht relevant.",
        talk_id: None,
    },
    HotspotSpec {
        id: "route_map",
        name: "Gebäudeplan",
        pct: pct(92.8, 31.69, 6.51, 21.64),
        kind: HotspotKind::Prop,
        look: "Ein Plan der Häuser 9, 11 und 15.",
        inspect: "Studio, Serverraum und Szenenbau sind eingezeichnet. Der Außenstandort ist nicht Teil des Gebäudeplans.",
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
        look: "Die Kreuzschiene verteilt die Signale durch Räume und Studios.",
        inspect: "Die Route von Nummer 11 ins Studio ist aktiv. Sie braucht ein lesbares Label.",
        talk_id: None,
    },
    HotspotSpec {
        id: "sdi_spool",
        name: "80-m-SDI-Rolle",
        pct: pct(70.59, 51.56, 13.15, 23.97),
        kind: HotspotKind::Prop,
        look: "Eine schwere Rolle SDI-Kabel liegt am Boden.",
        inspect:
            "Die Kabellänge reicht bis ins Studio. Die Strecke muss eindeutig beschriftet werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "sdi_label_printer",
        name: "Labeldrucker",
        pct: pct(62.8, 41.41, 5.2, 9.53),
        kind: HotspotKind::Pickup,
        look: "Ein kleiner Labeldrucker steht neben dem Patchfeld.",
        inspect: "Das vorbereitete Etikett benennt die Greenscreen-Route.",
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
        inspect: "Die Kulissenteile sind beschriftet und eingelagert. Sie werden für diese Greenscreen-Probe nicht verwendet.",
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
        name: "Probenmonitor",
        pct: pct(32.57, 46.47, 3.46, 7.31),
        kind: HotspotKind::Prop,
        look: "Der Monitor zeigt das Studio und einen leeren Hintergrundkanal.",
        inspect: "Bild und Ton liegen an. Für den Hintergrund fehlen Grafikdaten und Referenzen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "graphics_terminal",
        name: "Grafikplatz",
        pct: pct(67.72, 42.47, 11.03, 26.81),
        kind: HotspotKind::Prop,
        look: "Der Grafikplatz ist für Bauchbinden und Einblendungen vorbereitet.",
        inspect: "Im System ist ein leerer Platzhalter. Die freigegebene Bauchbindenkarte fehlt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "intercom_voice",
        name: "Regiestimme",
        pct: pct(57.15, 45.89, 5.99, 13.89),
        kind: HotspotKind::Character,
        look: "Eine Stimme aus der Gegensprechanlage.",
        inspect: "Die Gegensprechanlage ist offen. Die Regie wartet auf den vollständigen Probelauf.",
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
        id: "video_kliemannsland_road",
        name: "Straßenvideo",
        pct: pct(67.84, 15.17, 10.98, 16.69),
        kind: HotspotKind::Exit,
        look: "Ein Monitor zeigt einen Straßenframe aus dem Videoarchiv.",
        inspect: "Der Frame enthält eine erkennbare Standfläche. Er kann als begehbarer Videoraum geöffnet werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "video_schimmelbrueder",
        name: "Schimmelbrüder-Video",
        pct: pct(63.58, 33.36, 3.9, 7.94),
        kind: HotspotKind::Exit,
        look: "Ein Monitor zeigt eine Fertigungshalle mit langen Formenreihen.",
        inspect: "Der Frame enthält Hallenboden, Gussformen und eine markante Musterbahn. Die Figur kann dort stehen.",
        talk_id: None,
    },
];

const VIDEO_ROAD_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "control_room",
        name: "Zurück zur Regie",
        pct: pct(3.0, 62.0, 10.0, 25.0),
        kind: HotspotKind::Exit,
        look: "Der Rückweg zur Regie liegt am linken Bildrand.",
        inspect: "Die Verbindung zur Regie ist aktiv.",
        talk_id: None,
    },
    HotspotSpec {
        id: "walkable_lane",
        name: "Standspur",
        pct: pct(49.0, 73.0, 24.0, 18.0),
        kind: HotspotKind::Prop,
        look: "Am Straßenrand liegt eine helle, begehbare Spur.",
        inspect: "Die Spur ist breit genug für die Figur und führt bis zur rechten Fahrspur.",
        talk_id: None,
    },
    HotspotSpec {
        id: "road_marker",
        name: "Routenmarker",
        pct: pct(36.0, 56.0, 11.0, 23.0),
        kind: HotspotKind::Pickup,
        look: "Ein Marker liegt auf der begehbaren Spur.",
        inspect: "Der Marker ist dem Straßenframe zugeordnet und kann als Referenz für den Ausgang dienen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "traffic_totem",
        name: "Markierungsgruppe",
        pct: pct(43.0, 35.0, 18.0, 37.0),
        kind: HotspotKind::Prop,
        look: "Mehrere rot-weiße Markierungen stehen entlang der Fahrspur.",
        inspect: "Die Markierungen ordnen den Frame nach Vordergrund, Mittelgrund und Hintergrund.",
        talk_id: None,
    },
    HotspotSpec {
        id: "archive_exit_sign",
        name: "Ausfahrtschild",
        pct: pct(72.0, 48.0, 16.0, 24.0),
        kind: HotspotKind::Prop,
        look: "Ein dunkles Schild liegt über der rechten Fahrspur.",
        inspect: "Der Ausgang ist an den Marker aus diesem Straßenframe gekoppelt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "distant_gate",
        name: "Straßenende",
        pct: pct(83.0, 39.0, 12.0, 30.0),
        kind: HotspotKind::Prop,
        look: "Am rechten Bildrand endet die begehbare Straßenfläche.",
        inspect: "Der Anschluss zum nächsten Clip ist noch gesperrt.",
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
        look: "Eine lange Arbeitsfläche zieht sich in den linken Bildrand.",
        inspect: "Die roten Markierungen kennzeichnen wiederkehrende Arbeitsschritte an der Gusslinie.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mold_rack",
        name: "Formenreihe",
        pct: pct(19.61, 27.44, 21.06, 39.05),
        kind: HotspotKind::Prop,
        look: "Mehrere runde Formen stehen in zwei Reihen.",
        inspect: "Die Formen unterscheiden sich in Füllstand und Helligkeit. Die Reihenfolge läuft von links nach rechts.",
        talk_id: None,
    },
    HotspotSpec {
        id: "patterned_belt",
        name: "Musterbahn",
        pct: pct(55.0, 17.0, 31.0, 42.0),
        kind: HotspotKind::Prop,
        look: "Auf der rechten Bahn liegt ein helles Zickzackmuster.",
        inspect: "Das Muster entspricht der Reihenfolge der Formen: leer, voll, leer, voll.",
        talk_id: None,
    },
    HotspotSpec {
        id: "mold_token",
        name: "Formprobe",
        pct: pct(41.3, 80.56, 8.97, 13.22),
        kind: HotspotKind::Pickup,
        look: "Ein einzelnes helles Stück liegt am Rand der Gusslinie.",
        inspect: "Die Probe ist ein bewegliches Teil aus der Formenreihe und kann mitgenommen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "song_exit_gate",
        name: "Clip-Ausgang",
        pct: pct(86.48, 29.36, 13.35, 53.11),
        kind: HotspotKind::Prop,
        look: "Am rechten Rand liegt der Ausgang aus dem geöffneten Clip.",
        inspect: "Der Ausgang ist mit der Musterbahn und der Formprobe verknüpft.",
        talk_id: None,
    },
    HotspotSpec {
        id: "factory_floor",
        name: "Hallenboden",
        pct: pct(7.57, 66.0, 79.24, 34.0),
        kind: HotspotKind::Prop,
        look: "Der Hallenboden ist zwischen Formenreihe und Musterbahn begehbar.",
        inspect: "Der Frame enthält eine durchgehende Standfläche vor der Formenreihe.",
        talk_id: None,
    },
];

const VIDEO_SEWER_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "control_room",
        name: "Zurück zur Regie",
        pct: pct(4.0, 65.0, 10.0, 22.0),
        kind: HotspotKind::Exit,
        look: "Der Rückweg zur Regie liegt am linken unteren Bildrand.",
        inspect: "Die Verbindung zur Regie ist aktiv.",
        talk_id: None,
    },
    HotspotSpec {
        id: "archive_hatch",
        name: "Archivluke",
        pct: pct(72.0, 40.0, 13.0, 32.0),
        kind: HotspotKind::Prop,
        look: "Eine runde Luke sitzt in der Rohrwand.",
        inspect: "Die Luke ist mit einem Timecode-Schloss versehen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "flow_arrow",
        name: "Flusspfeil",
        pct: pct(42.0, 73.0, 13.0, 8.0),
        kind: HotspotKind::Prop,
        look: "Ein Pfeil ist auf dem Boden markiert.",
        inspect: "Die Pfeilrichtung definiert die Reihenfolge: Zulauf, Ablauf, Gegenlauf.",
        talk_id: None,
    },
    HotspotSpec {
        id: "inspection_tripod",
        name: "Inspektionsstativ",
        pct: pct(19.0, 44.0, 11.0, 34.0),
        kind: HotspotKind::Prop,
        look: "Ein Inspektionsstativ steht am linken Rand.",
        inspect: "Die Optik ist auf die Luke ausgerichtet.",
        talk_id: None,
    },
    HotspotSpec {
        id: "wet_note",
        name: "Nasser Zettel",
        pct: pct(58.5, 80.0, 5.5, 5.5),
        kind: HotspotKind::Pickup,
        look: "Ein Stück Papier klebt im Schmutzwasser.",
        inspect: "Die Tinte ist verlaufen. Nur ein Timecode und drei Pfeile sind noch lesbar.",
        talk_id: None,
    },
    HotspotSpec {
        id: "lost_signal",
        name: "Fehlender Bildbereich",
        pct: pct(43.0, 20.0, 22.0, 32.0),
        kind: HotspotKind::Prop,
        look: "In der Bildmitte fehlt ein Teil des Videoframes.",
        inspect: "Der fehlende Bereich verweist auf einen nicht geladenen Frame im Archiv.",
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
        inspect: "Die Bahnhofsuhr kann als Zeitreferenz für den Probelauf dienen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "karoviertel",
        name: "Karoviertel",
        pct: pct(37.38, 27.78, 14.0, 42.92),
        kind: HotspotKind::Exit,
        look: "Ein Abzweig führt Richtung Karoviertel.",
        inspect: "Der Laufzettel nennt dort eine Druckfreigabe.",
        talk_id: None,
    },
    HotspotSpec {
        id: "street_mural",
        name: "Wandbild",
        pct: pct(12.47, 4.28, 20.47, 56.33),
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
        look: "Ein Kiosk steht an der Kreuzung.",
        inspect: "Der Kiosk ist geschlossen und für den Probelauf nicht relevant.",
        talk_id: None,
    },
    HotspotSpec {
        id: "city_notice",
        name: "Drehhinweis",
        pct: pct(32.73, 31.86, 3.05, 12.25),
        kind: HotspotKind::Prop,
        look: "Ein schmaler Aushang klebt am Laternenmast.",
        inspect: "Der Aushang nennt Timing, Grafikfreigabe und Lichtreferenz.",
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
        look: "Die Bahnhofsuhr hängt über dem Durchgang.",
        inspect: "Die Uhr liefert eine eindeutige Zeitmarke für die Außenreferenz.",
        talk_id: None,
    },
    HotspotSpec {
        id: "platform_sign",
        name: "Bahnsteigschild",
        pct: pct(65.74, 31.95, 7.34, 8.5),
        kind: HotspotKind::Prop,
        look: "Ein Schild weist zum Bahnsteig.",
        inspect:
            "Das Schild bestätigt die Richtung zum Bahnsteig. Für den Probelauf wird nur die Uhr benötigt.",
        talk_id: None,
    },
    HotspotSpec {
        id: "busker_case",
        name: "Straßenmusiker-Koffer",
        pct: pct(31.07, 57.42, 7.21, 17.33),
        kind: HotspotKind::Character,
        look: "Ein offener Koffer liegt vor einem Straßenmusiker.",
        inspect:
            "Der Musiker steht am Durchgang. Er kennt die Wege zwischen Bahnhof und Park.",
        talk_id: Some("busker"),
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
        look: "Der Wasserturm steht oberhalb des Parks.",
        inspect:
            "Die Backsteinfläche liefert eine klare Lichtreferenz für den Greenscreen-Hintergrund.",
        talk_id: None,
    },
    HotspotSpec {
        id: "city_reflector",
        name: "Lichtreflektor",
        pct: pct(64.37, 62.22, 5.53, 13.64),
        kind: HotspotKind::Pickup,
        look: "Ein kleiner Reflektor liegt neben dem Weg.",
        inspect: "Der Reflektor kann die Lichtreferenz für das Studio übernehmen.",
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
        look: "Im Copyshop liegen Farbfächer und frisch geschnittene Karten.",
        inspect: "Eine freigegebene Karte für die Bauchbinde liegt am Tresen.",
        talk_id: None,
    },
    HotspotSpec {
        id: "record_store",
        name: "Plattenladen",
        pct: pct(65.62, 29.06, 18.86, 36.5),
        kind: HotspotKind::Prop,
        look: "Ein kleiner Laden mit Plakaten im Fenster.",
        inspect: "Der Laden ist nicht Teil des aktuellen Laufzettels.",
        talk_id: None,
    },
    HotspotSpec {
        id: "market_boxes",
        name: "Marktkisten",
        pct: pct(16.34, 59.78, 14.0, 16.25),
        kind: HotspotKind::Prop,
        look: "Kisten und Kartons stehen am Rand des Gehwegs.",
        inspect: "Die Kisten gehören zu einem anderen Geschäft und sind nicht relevant.",
        talk_id: None,
    },
];

const HOSPITAL_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "hospital_monitor",
        name: "Überwachungsmonitor",
        pct: pct(35.0, 20.0, 9.0, 24.0),
        kind: HotspotKind::Prop,
        look: "Der Monitor zeigt stabile Werte. Der Stromschlag war real, aber nicht endgültig.",
        inspect: "Puls, Sauerstoff und EKG sind wieder im grünen Bereich.",
        talk_id: None,
    },
    HotspotSpec {
        id: "call_button",
        name: "Rufknopf",
        pct: pct(44.0, 55.0, 5.0, 10.0),
        kind: HotspotKind::Prop,
        look: "Ein roter Rufknopf hängt am Bett.",
        inspect: "Ohne Pflegerückmeldung kommst du nicht aus dem Zimmer.",
        talk_id: None,
    },
    HotspotSpec {
        id: "nurse_station",
        name: "Pflegekraft",
        pct: pct(69.0, 24.0, 9.0, 35.0),
        kind: HotspotKind::Character,
        look: "Die Pflegekraft prüft Akte und Monitor.",
        inspect: "Sie wartet auf klare Werte und eine sachliche Erklärung.",
        talk_id: Some("nurse"),
    },
    HotspotSpec {
        id: "discharge_clipboard",
        name: "Entlassungsbogen",
        pct: pct(54.0, 50.0, 8.0, 12.0),
        kind: HotspotKind::Pickup,
        look: "Ein Formular liegt auf dem Nachttisch.",
        inspect: "Der Bogen darf erst mit stabilen Werten und Rücksprache mitgenommen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "hospital_exit",
        name: "Korridortür",
        pct: pct(88.0, 19.0, 8.0, 49.0),
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
        pct: pct(18.0, 30.0, 7.0, 24.0),
        kind: HotspotKind::Prop,
        look: "Das Alarmfeld protokolliert den Sprinklerlauf.",
        inspect: "Der ausgelöste Kreis muss bestätigt werden, bevor jemand zurück in den Bau darf.",
        talk_id: None,
    },
    HotspotSpec {
        id: "extinguisher_cabinet",
        name: "Löschschrank",
        pct: pct(40.0, 39.0, 8.0, 19.0),
        kind: HotspotKind::Prop,
        look: "Der Schrank ist geöffnet, aber vollständig.",
        inspect: "Es fehlt kein Löscher. Der Schaden ist Wasser, nicht Feuer.",
        talk_id: None,
    },
    HotspotSpec {
        id: "safety_officer",
        name: "Sicherheitsdienst",
        pct: pct(63.0, 34.0, 8.0, 32.0),
        kind: HotspotKind::Character,
        look: "Der Sicherheitsdienst schreibt den Vorfall auf.",
        inspect: "Er lässt dich erst nach technischer Rückmeldung zurück.",
        talk_id: Some("safety"),
    },
    HotspotSpec {
        id: "fire_return_door",
        name: "Rückweg ins Gebäude",
        pct: pct(86.0, 24.0, 9.0, 45.0),
        kind: HotspotKind::Prop,
        look: "Die Tür zurück zum Produktionsgebäude ist nass, aber offen.",
        inspect: "Der Rückweg ist erst nach Alarm- und Materialprüfung frei.",
        talk_id: None,
    },
];

const PROP_STORAGE_COLLAPSE_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "brace_beam",
        name: "Stützstrebe",
        pct: pct(38.0, 29.0, 13.0, 28.0),
        kind: HotspotKind::Prop,
        look: "Eine Strebe hält die gefallenen Requisiten gerade noch.",
        inspect: "Wenn sie korrekt sitzt, kann der Durchgang freigeräumt werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "release_rope",
        name: "Notleine",
        pct: pct(59.0, 18.0, 5.0, 40.0),
        kind: HotspotKind::Prop,
        look: "Eine Leine hängt an der oberen Traverse.",
        inspect: "Sie bewegt einen Riegel über dem Durchgang.",
        talk_id: None,
    },
    HotspotSpec {
        id: "workshop_gap",
        name: "Freier Spalt",
        pct: pct(70.0, 55.0, 10.0, 20.0),
        kind: HotspotKind::Prop,
        look: "Zwischen den Teilen bleibt ein schmaler Spalt.",
        inspect: "Der Spalt reicht nur, wenn die Strebe hält und der Riegel offen ist.",
        talk_id: None,
    },
    HotspotSpec {
        id: "collapse_exit",
        name: "Werkstatttür",
        pct: pct(88.0, 28.0, 8.0, 42.0),
        kind: HotspotKind::Prop,
        look: "Die Tür führt zurück in den Szenenbau.",
        inspect: "Der direkte Weg ist von Kulissenteilen blockiert.",
        talk_id: None,
    },
];

const ARCHIVE_RECOVERY_HOTSPOTS: &[HotspotSpec] = &[
    HotspotSpec {
        id: "recovery_terminal",
        name: "Rettungsterminal",
        pct: pct(43.0, 28.0, 13.0, 28.0),
        kind: HotspotKind::Prop,
        look: "Das Terminal zeigt die letzte lesbare Frame-ID.",
        inspect: "Die Session muss mit Prüfsumme abgeschlossen werden.",
        talk_id: None,
    },
    HotspotSpec {
        id: "checksum_printout",
        name: "Prüfsummenzettel",
        pct: pct(58.0, 54.0, 7.0, 9.0),
        kind: HotspotKind::Pickup,
        look: "Ein Ausdruck liegt neben der Tastatur.",
        inspect: "Die Prüfsumme passt zum beschädigten Videoframe.",
        talk_id: None,
    },
    HotspotSpec {
        id: "archive_drive",
        name: "Archivlaufwerk",
        pct: pct(22.0, 28.0, 12.0, 32.0),
        kind: HotspotKind::Prop,
        look: "Das Laufwerk klackert in kurzen, gleichmäßigen Abständen.",
        inspect: "Die Medien sind lesbar, aber die Session ist noch nicht quittiert.",
        talk_id: None,
    },
    HotspotSpec {
        id: "control_room_return",
        name: "Tür zur Regie",
        pct: pct(88.0, 21.0, 8.0, 47.0),
        kind: HotspotKind::Prop,
        look: "Die Tür führt zurück zur Regie.",
        inspect: "Der Rückweg ist erst nach einer sauber abgeschlossenen Rettungssession frei.",
        talk_id: None,
    },
];

const HOTSPOT_POLYGONS: &[HotspotPolygonSpec] = &[
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "office_hall",
        points: &[(5.43, 25.47), (11.52, 25.47), (11.52, 68.39), (5.43, 68.39)],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "control_room",
        points: &[
            (94.43, 26.61),
            (99.28, 26.61),
            (99.28, 78.44),
            (94.35, 74.9),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "greenscreen_wall",
        points: &[
            (38.65, 16.75),
            (71.41, 16.75),
            (71.41, 63.97),
            (38.65, 63.97),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "floor_marks",
        points: &[
            (38.87, 63.25),
            (60.87, 63.25),
            (60.87, 72.25),
            (38.87, 72.25),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "klixx_table",
        points: &[(23.54, 51.11), (40.22, 51.11), (40.22, 75.5), (23.54, 75.5)],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "chat_preview",
        points: &[
            (19.94, 29.56),
            (24.51, 29.56),
            (24.51, 47.92),
            (19.94, 47.92),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "camera_one",
        points: &[
            (62.34, 41.44),
            (73.09, 41.44),
            (73.09, 76.13),
            (62.34, 76.13),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "mentor_shadow",
        points: &[
            (75.74, 37.92),
            (80.41, 37.92),
            (80.41, 72.17),
            (75.74, 72.17),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "greenscreen_studio",
        hotspot_id: "greenscreen_setup",
        points: &[
            (49.18, 35.89),
            (72.18, 35.89),
            (72.18, 72.89),
            (49.18, 72.89),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "building_courtyard",
        points: &[(2.49, 21.81), (9.07, 21.81), (9.07, 68.67), (2.49, 68.67)],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "greenscreen_studio",
        points: &[
            (16.33, 25.78),
            (23.25, 25.78),
            (23.25, 67.45),
            (16.33, 67.45),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "control_room",
        points: &[
            (34.16, 26.25),
            (40.41, 26.25),
            (40.41, 67.14),
            (34.16, 67.14),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "server_room",
        points: &[(84.65, 27.42), (89.7, 27.42), (89.7, 67.98), (84.65, 67.98)],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "schanzenstrasse",
        points: &[(73.18, 27.39), (79.1, 27.39), (79.1, 64.7), (73.18, 64.7)],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "dispo_board",
        points: &[
            (24.53, 31.94),
            (32.55, 31.94),
            (32.55, 54.61),
            (24.53, 54.61),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "equipment_storage",
        points: &[
            (43.54, 26.25),
            (56.54, 26.25),
            (56.54, 67.08),
            (43.54, 67.08),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "staircase",
        points: &[(60.8, 13.11), (69.78, 13.11), (69.78, 63.17), (60.8, 63.17)],
    },
    HotspotPolygonSpec {
        scene_id: "office_hall",
        hotspot_id: "route_map",
        points: &[(92.8, 31.69), (99.31, 31.69), (99.31, 53.33), (92.8, 53.33)],
    },
    HotspotPolygonSpec {
        scene_id: "building_courtyard",
        hotspot_id: "office_hall",
        points: &[
            (50.44, 31.58),
            (57.89, 31.58),
            (57.89, 63.36),
            (50.44, 63.36),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "building_courtyard",
        hotspot_id: "set_workshop",
        points: &[(80.72, 33.5), (85.89, 33.5), (85.89, 67.42), (80.72, 67.42)],
    },
    HotspotPolygonSpec {
        scene_id: "building_courtyard",
        hotspot_id: "schanzenstrasse",
        points: &[(7.67, 25.97), (15.42, 25.97), (15.42, 73.3), (7.67, 73.3)],
    },
    HotspotPolygonSpec {
        scene_id: "building_courtyard",
        hotspot_id: "loading_zone",
        points: &[
            (19.56, 46.33),
            (37.56, 46.33),
            (37.56, 72.02),
            (19.56, 72.02),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "building_courtyard",
        hotspot_id: "address_plate",
        points: &[
            (68.55, 30.78),
            (72.59, 30.78),
            (72.59, 43.59),
            (68.55, 43.59),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "office_hall",
        points: &[(3.39, 23.92), (10.72, 23.92), (10.72, 68.56), (3.39, 68.56)],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "greenscreen_studio",
        points: &[
            (22.16, 15.14),
            (46.91, 15.14),
            (46.91, 44.03),
            (22.16, 44.03),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "rehearsal_monitor",
        points: &[
            (32.57, 46.47),
            (36.03, 46.47),
            (36.03, 53.78),
            (32.57, 53.78),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "graphics_terminal",
        points: &[
            (67.72, 42.47),
            (78.75, 42.47),
            (78.75, 69.28),
            (67.72, 69.28),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "intercom_voice",
        points: &[
            (57.15, 45.89),
            (63.14, 45.89),
            (63.14, 59.78),
            (57.15, 59.78),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "on_air_lamp",
        points: &[(87.66, 10.25), (92.0, 10.25), (92.0, 16.25), (87.66, 16.25)],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "video_kliemannsland_road",
        points: &[
            (67.84, 15.17),
            (78.82, 15.17),
            (78.82, 31.86),
            (67.84, 31.86),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "control_room",
        hotspot_id: "video_schimmelbrueder",
        points: &[(63.58, 33.36), (67.48, 33.36), (67.48, 41.3), (63.58, 41.3)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "control_room",
        points: &[(3.0, 62.0), (13.0, 62.0), (13.0, 87.0), (3.0, 87.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "walkable_lane",
        points: &[(49.0, 73.0), (73.0, 73.0), (73.0, 91.0), (49.0, 91.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "road_marker",
        points: &[(36.0, 56.0), (47.0, 56.0), (47.0, 79.0), (36.0, 79.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "traffic_totem",
        points: &[(43.0, 35.0), (61.0, 35.0), (61.0, 72.0), (43.0, 72.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "archive_exit_sign",
        points: &[(72.0, 48.0), (88.0, 48.0), (88.0, 72.0), (72.0, 72.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_kliemannsland_road",
        hotspot_id: "distant_gate",
        points: &[(83.0, 39.0), (95.0, 39.0), (95.0, 69.0), (83.0, 69.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "control_room",
        points: &[(1.32, 21.94), (7.94, 21.94), (7.94, 38.19), (1.32, 38.19)],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "casting_table",
        points: &[
            (26.61, 21.7),
            (40.11, 24.06),
            (16.56, 55.31),
            (8.87, 54.34),
            (9.14, 31.56),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "mold_rack",
        points: &[
            (50.38, 18.37),
            (57.15, 19.48),
            (40.67, 66.49),
            (19.61, 66.49),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "patterned_belt",
        points: &[
            (61.56, 12.12),
            (72.2, 11.56),
            (85.81, 57.67),
            (54.57, 58.23),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "mold_token",
        points: &[(41.3, 80.56), (50.27, 80.56), (50.27, 93.78), (41.3, 93.78)],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "song_exit_gate",
        points: &[
            (86.48, 29.36),
            (99.83, 29.36),
            (99.83, 82.47),
            (86.48, 82.47),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "video_schimmelbrueder",
        hotspot_id: "factory_floor",
        points: &[(7.57, 66.0), (86.81, 66.0), (86.81, 100.0), (7.57, 100.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "control_room",
        points: &[(4.0, 65.0), (14.0, 65.0), (14.0, 87.0), (4.0, 87.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "archive_hatch",
        points: &[(72.0, 40.0), (85.0, 40.0), (85.0, 72.0), (72.0, 72.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "flow_arrow",
        points: &[(42.0, 73.0), (55.0, 73.0), (55.0, 81.0), (42.0, 81.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "inspection_tripod",
        points: &[(19.0, 44.0), (30.0, 44.0), (30.0, 78.0), (19.0, 78.0)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "wet_note",
        points: &[(58.5, 80.0), (64.0, 80.0), (64.0, 85.5), (58.5, 85.5)],
    },
    HotspotPolygonSpec {
        scene_id: "video_sewer_archive",
        hotspot_id: "lost_signal",
        points: &[(43.0, 20.0), (65.0, 20.0), (65.0, 52.0), (43.0, 52.0)],
    },
    HotspotPolygonSpec {
        scene_id: "server_room",
        hotspot_id: "office_hall",
        points: &[(4.35, 25.69), (9.86, 25.69), (9.86, 71.58), (4.35, 71.58)],
    },
    HotspotPolygonSpec {
        scene_id: "server_room",
        hotspot_id: "server_racks",
        points: &[(17.17, 19.3), (39.04, 19.3), (39.04, 72.16), (17.17, 72.16)],
    },
    HotspotPolygonSpec {
        scene_id: "server_room",
        hotspot_id: "video_hub",
        points: &[
            (41.82, 21.31),
            (61.16, 21.31),
            (61.16, 70.98),
            (41.82, 70.98),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "server_room",
        hotspot_id: "sdi_spool",
        points: &[
            (70.59, 51.56),
            (83.74, 51.56),
            (83.74, 75.53),
            (70.59, 75.53),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "server_room",
        hotspot_id: "sdi_label_printer",
        points: &[(62.8, 41.41), (68.0, 41.41), (68.0, 50.94), (62.8, 50.94)],
    },
    HotspotPolygonSpec {
        scene_id: "set_workshop",
        hotspot_id: "building_courtyard",
        points: &[(4.19, 27.92), (9.21, 27.92), (9.21, 72.7), (4.19, 72.7)],
    },
    HotspotPolygonSpec {
        scene_id: "set_workshop",
        hotspot_id: "set_pieces",
        points: &[
            (14.65, 13.39),
            (38.65, 13.39),
            (38.65, 70.28),
            (14.65, 70.28),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "set_workshop",
        hotspot_id: "gaffer_roll",
        points: &[(57.0, 51.33), (60.27, 51.33), (60.27, 57.5), (57.0, 57.5)],
    },
    HotspotPolygonSpec {
        scene_id: "set_workshop",
        hotspot_id: "bike_workshop_trace",
        points: &[
            (67.49, 13.28),
            (83.49, 13.28),
            (83.49, 35.28),
            (67.49, 35.28),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "set_workshop",
        hotspot_id: "studio_door_15",
        points: &[
            (85.53, 29.31),
            (93.01, 29.31),
            (93.01, 67.28),
            (85.53, 67.28),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "building_courtyard",
        points: &[(3.12, 38.39), (10.1, 38.39), (10.1, 70.89), (3.12, 70.89)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "sternschanze_station",
        points: &[
            (73.14, 31.94),
            (100.0, 31.94),
            (100.0, 71.11),
            (73.14, 71.11),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "karoviertel",
        points: &[(37.38, 27.78), (51.38, 27.78), (51.38, 70.7), (37.38, 70.7)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "street_mural",
        points: &[(12.47, 4.28), (32.94, 4.28), (32.94, 60.61), (12.47, 60.61)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "corner_kiosk",
        points: &[
            (58.48, 41.11),
            (69.48, 41.11),
            (69.48, 61.67),
            (58.48, 61.67),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenstrasse",
        hotspot_id: "city_notice",
        points: &[
            (32.73, 31.86),
            (35.78, 31.86),
            (35.78, 44.11),
            (32.73, 44.11),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "sternschanze_station",
        hotspot_id: "schanzenstrasse",
        points: &[(6.31, 28.86), (20.31, 28.86), (20.31, 69.08), (6.31, 69.08)],
    },
    HotspotPolygonSpec {
        scene_id: "sternschanze_station",
        hotspot_id: "schanzenpark",
        points: &[
            (79.88, 32.63),
            (95.44, 32.63),
            (95.44, 64.44),
            (79.88, 64.44),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "sternschanze_station",
        hotspot_id: "station_clock",
        points: &[(41.8, 22.44), (45.99, 22.44), (45.99, 34.61), (41.8, 34.61)],
    },
    HotspotPolygonSpec {
        scene_id: "sternschanze_station",
        hotspot_id: "platform_sign",
        points: &[
            (65.74, 31.95),
            (73.08, 31.95),
            (73.08, 40.45),
            (65.74, 40.45),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "sternschanze_station",
        hotspot_id: "busker_case",
        points: &[
            (31.07, 57.42),
            (38.28, 57.42),
            (38.28, 74.75),
            (31.07, 74.75),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenpark",
        hotspot_id: "sternschanze_station",
        points: &[(5.54, 53.33), (25.94, 53.33), (25.94, 74.36), (5.54, 74.36)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenpark",
        hotspot_id: "schanzenstrasse",
        points: &[(5.9, 75.11), (18.9, 75.11), (18.9, 91.11), (5.9, 91.11)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenpark",
        hotspot_id: "water_tower",
        points: &[(36.46, 6.61), (46.77, 6.61), (46.77, 54.97), (36.46, 54.97)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenpark",
        hotspot_id: "city_reflector",
        points: &[(64.37, 62.22), (69.9, 62.22), (69.9, 75.86), (64.37, 75.86)],
    },
    HotspotPolygonSpec {
        scene_id: "schanzenpark",
        hotspot_id: "tv_tower_view",
        points: &[
            (74.12, 24.45),
            (77.09, 24.45),
            (77.09, 58.28),
            (74.12, 58.28),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "karoviertel",
        hotspot_id: "schanzenstrasse",
        points: &[(7.71, 14.28), (21.71, 14.28), (21.71, 64.5), (7.71, 64.5)],
    },
    HotspotPolygonSpec {
        scene_id: "karoviertel",
        hotspot_id: "print_shop",
        points: &[
            (31.78, 17.03),
            (52.79, 17.03),
            (52.79, 64.67),
            (31.78, 64.67),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "karoviertel",
        hotspot_id: "record_store",
        points: &[
            (65.62, 29.06),
            (84.48, 29.06),
            (84.48, 65.56),
            (65.62, 65.56),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "karoviertel",
        hotspot_id: "market_boxes",
        points: &[
            (16.34, 59.78),
            (30.34, 59.78),
            (30.34, 76.03),
            (16.34, 76.03),
        ],
    },
    HotspotPolygonSpec {
        scene_id: "hospital_room",
        hotspot_id: "hospital_monitor",
        points: &[(35.0, 20.0), (44.0, 20.0), (44.0, 44.0), (35.0, 44.0)],
    },
    HotspotPolygonSpec {
        scene_id: "hospital_room",
        hotspot_id: "call_button",
        points: &[(44.0, 55.0), (49.0, 55.0), (49.0, 65.0), (44.0, 65.0)],
    },
    HotspotPolygonSpec {
        scene_id: "hospital_room",
        hotspot_id: "nurse_station",
        points: &[(69.0, 24.0), (78.0, 24.0), (78.0, 59.0), (69.0, 59.0)],
    },
    HotspotPolygonSpec {
        scene_id: "hospital_room",
        hotspot_id: "discharge_clipboard",
        points: &[(54.0, 50.0), (62.0, 50.0), (62.0, 62.0), (54.0, 62.0)],
    },
    HotspotPolygonSpec {
        scene_id: "hospital_room",
        hotspot_id: "hospital_exit",
        points: &[(88.0, 19.0), (96.0, 19.0), (96.0, 68.0), (88.0, 68.0)],
    },
    HotspotPolygonSpec {
        scene_id: "sprinkler_courtyard",
        hotspot_id: "alarm_panel",
        points: &[(18.0, 30.0), (25.0, 30.0), (25.0, 54.0), (18.0, 54.0)],
    },
    HotspotPolygonSpec {
        scene_id: "sprinkler_courtyard",
        hotspot_id: "extinguisher_cabinet",
        points: &[(40.0, 39.0), (48.0, 39.0), (48.0, 58.0), (40.0, 58.0)],
    },
    HotspotPolygonSpec {
        scene_id: "sprinkler_courtyard",
        hotspot_id: "safety_officer",
        points: &[(63.0, 34.0), (71.0, 34.0), (71.0, 66.0), (63.0, 66.0)],
    },
    HotspotPolygonSpec {
        scene_id: "sprinkler_courtyard",
        hotspot_id: "fire_return_door",
        points: &[(86.0, 24.0), (95.0, 24.0), (95.0, 69.0), (86.0, 69.0)],
    },
    HotspotPolygonSpec {
        scene_id: "prop_storage_collapse",
        hotspot_id: "brace_beam",
        points: &[(38.0, 29.0), (51.0, 29.0), (51.0, 57.0), (38.0, 57.0)],
    },
    HotspotPolygonSpec {
        scene_id: "prop_storage_collapse",
        hotspot_id: "release_rope",
        points: &[(59.0, 18.0), (64.0, 18.0), (64.0, 58.0), (59.0, 58.0)],
    },
    HotspotPolygonSpec {
        scene_id: "prop_storage_collapse",
        hotspot_id: "workshop_gap",
        points: &[(70.0, 55.0), (80.0, 55.0), (80.0, 75.0), (70.0, 75.0)],
    },
    HotspotPolygonSpec {
        scene_id: "prop_storage_collapse",
        hotspot_id: "collapse_exit",
        points: &[(88.0, 28.0), (96.0, 28.0), (96.0, 70.0), (88.0, 70.0)],
    },
    HotspotPolygonSpec {
        scene_id: "archive_recovery",
        hotspot_id: "archive_drive",
        points: &[(22.0, 28.0), (34.0, 28.0), (34.0, 60.0), (22.0, 60.0)],
    },
    HotspotPolygonSpec {
        scene_id: "archive_recovery",
        hotspot_id: "recovery_terminal",
        points: &[(43.0, 28.0), (56.0, 28.0), (56.0, 56.0), (43.0, 56.0)],
    },
    HotspotPolygonSpec {
        scene_id: "archive_recovery",
        hotspot_id: "checksum_printout",
        points: &[(58.0, 54.0), (65.0, 54.0), (65.0, 63.0), (58.0, 63.0)],
    },
    HotspotPolygonSpec {
        scene_id: "archive_recovery",
        hotspot_id: "control_room_return",
        points: &[(88.0, 21.0), (96.0, 21.0), (96.0, 68.0), (88.0, 68.0)],
    },
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
const VIDEO_ROAD_WALKABLE: &[(f32, f32)] = &[
    (24.0, 45.0),
    (58.0, 47.0),
    (90.0, 96.0),
    (3.0, 96.0),
    (10.0, 72.0),
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
const VIDEO_SEWER_WALKABLE: &[(f32, f32)] =
    &[(15.0, 72.0), (85.0, 72.0), (94.0, 96.0), (7.0, 96.0)];
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
    (9.0, 86.0),
    (18.0, 69.0),
    (42.0, 66.0),
    (52.0, 72.0),
    (83.0, 71.0),
    (96.0, 84.0),
    (100.0, 100.0),
    (0.0, 100.0),
];
const SPRINKLER_COURTYARD_WALKABLE: &[(f32, f32)] = &[
    (0.0, 77.0),
    (20.0, 66.0),
    (64.0, 67.0),
    (100.0, 76.0),
    (100.0, 100.0),
    (0.0, 100.0),
];
const PROP_STORAGE_COLLAPSE_WALKABLE: &[(f32, f32)] = &[
    (6.0, 88.0),
    (18.0, 72.0),
    (48.0, 70.0),
    (67.0, 78.0),
    (96.0, 77.0),
    (100.0, 100.0),
    (0.0, 100.0),
];
const ARCHIVE_RECOVERY_WALKABLE: &[(f32, f32)] = &[
    (0.0, 82.0),
    (11.0, 68.0),
    (38.0, 67.0),
    (56.0, 72.0),
    (82.0, 68.0),
    (100.0, 78.0),
    (100.0, 100.0),
    (0.0, 100.0),
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
        id: "video_kliemannsland_road",
        name: "Straßenvideo",
        zone: "Videoarchiv",
        token: (22.0, 84.0),
        walkable: VIDEO_ROAD_WALKABLE,
        hotspots: VIDEO_ROAD_HOTSPOTS,
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
        id: "video_sewer_archive",
        name: "Kanalvideo",
        zone: "Videoarchiv",
        token: (20.0, 84.0),
        walkable: VIDEO_SEWER_WALKABLE,
        hotspots: VIDEO_SEWER_HOTSPOTS,
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

const ITEMS: &[ItemMeta] = &[
    ItemMeta {
        id: "call_sheet",
        name: "Laufzettel",
        short: "PLAN",
        description: "Der Tagesplan für Studio, Regie und Außenreferenzen.",
    },
    ItemMeta {
        id: "gaffer_tape",
        name: "Gaffer-Tape",
        short: "TAPE",
        description: "Eine Rolle für neue Bodenmarken im Studio.",
    },
    ItemMeta {
        id: "sdi_label",
        name: "SDI-Label",
        short: "SDI",
        description: "Ein Etikett für die Signalroute von Haus 11 ins Studio.",
    },
    ItemMeta {
        id: "lower_third_card",
        name: "Bauchbindenkarte",
        short: "GRAF",
        description: "Farbkarte und Textfreigabe für die Bauchbinde in der Regie.",
    },
    ItemMeta {
        id: "city_reflector",
        name: "Lichtreflektor",
        short: "LICH",
        description: "Eine Lichtreferenz aus dem Park für das Studio.",
    },
    ItemMeta {
        id: "wet_note",
        name: "Nasser Timecode",
        short: "TIME",
        description: "Ein verwischter Timecode aus dem Kanalvideo: Zulauf, Ablauf, Gegenlauf.",
    },
    ItemMeta {
        id: "road_marker",
        name: "Routenmarker",
        short: "ROAD",
        description: "Ein Marker aus dem Straßenvideo. Er zeigt, wo das Bild begehbar ist.",
    },
    ItemMeta {
        id: "mold_token",
        name: "Formprobe",
        short: "FORM",
        description:
            "Eine Probe aus der Schimmelbrüder-Halle. Sie gehört zur Musterfolge der Formenreihe.",
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
        label: "Was ist der Auftrag?",
        response: "Bereite den Probelauf vor. Du brauchst den Laufzettel, neue Bodenmarken, ein SDI-Label, die Bauchbindenkarte, eine Zeitreferenz von der Sternschanze und eine Lichtreferenz aus dem Park.",
        flag: "mentor_goal_hint",
        log: "Die Aufnahmeleitung verteilt den Auftrag: Studio markieren, Signal benennen, Grafik laden, Stadt referenzieren.",
    },
    DialogueChoice {
        label: "Warum raus in die Stadt?",
        response: "Der Hintergrund soll nicht frei erfunden werden. Die Regie braucht eine reale Zeitmarke, eine Lichtreferenz und eine freigegebene Grafikvorlage.",
        flag: "mentor_city_hint",
        log: "Die Stadt liefert Referenzen für Timing, Licht und Grafik.",
    },
    DialogueChoice {
        label: "Warum ich?",
        response: "Du bist als Praktikant eingeteilt und gerade verfügbar. Halte dich an den Laufzettel und dokumentiere die offenen Punkte.",
        flag: "mentor_intern_hint",
        log: "Der Praktikant übernimmt die offenen Laufwege für den Probelauf.",
    },
    DialogueChoice {
        label: "Wie starte ich den Probelauf?",
        response: "Wenn alle Punkte erledigt sind, starte das Setup im Greenscreen-Studio. Die Regie prüft dann Signal, Grafik und Licht.",
        flag: "mentor_rehearsal_hint",
        log: "Der Probelauf wird am Setup im Greenscreen-Studio gestartet.",
    },
];

const BUSKER_CHOICES: &[DialogueChoice] = &[
    DialogueChoice {
        label: "Welche Zeitreferenz passt?",
        response: "Nimm die Bahnhofsuhr über dem Durchgang. Sie ist im Bild klar lesbar und wiederholt sich im Minutenrhythmus.",
        flag: "busker_timing_hint",
        log: "Der Musiker verweist auf die Bahnhofsuhr als Zeitreferenz für die Außenprobe.",
    },
    DialogueChoice {
        label: "Kennst du den Weg zum Park?",
        response: "Geh vom Bahnhof bergauf zum Wasserturm. Die Backsteinfläche liefert eine eindeutige Lichtreferenz.",
        flag: "busker_park_hint",
        log: "Der Musiker empfiehlt das Parklicht am Wasserturm als Referenz.",
    },
    DialogueChoice {
        label: "Gibt es weitere Hinweise?",
        response: "Für deine Aufgabe reichen Uhr, Parklicht und die Druckfreigabe im Karoviertel. Mehr steht nicht auf dem Laufzettel.",
        flag: "busker_scope_hint",
        log: "Der Musiker bestätigt die Außenstationen: Bahnhof, Park und Karoviertel.",
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
        flag: "nurse_clearance",
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
        opening: "Die Aufnahmeleitung bleibt am Studioeingang stehen und prüft den Laufzettel.",
        choices: MENTOR_CHOICES,
    },
    Dialogue {
        id: "busker",
        speaker: "Straßenmusiker",
        opening: "Der Straßenmusiker steht am Durchgang und zeigt auf die Bahnhofsuhr.",
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
            status: "Die Regie hat einen Videoeintrag geöffnet. Der vermisste Host muss über begehbare Frames lokalisiert werden.".to_string(),
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
                        self.status = "Der Probelauf ist abgeschlossen. Die offenen Archivräume bleiben dokumentiert.".to_string();
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
            self.hover = Some("Probe neu starten".to_string());
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
                "archive_hatch" => self.use_hotspot(id),
                "archive_exit_sign" | "distant_gate" => self.use_hotspot(id),
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
            "traffic_totem" => self.set_flag_log(
                "road_order_checked",
                "Die Markierungsgruppe reagiert auf den Stoß: nah, mittel, fern.",
                "Die Markierungen sind jetzt nach Bildtiefe sortiert.",
            ),
            "mold_rack" | "casting_table" | "set_pieces" => self.trigger_consequence(
                DeathKind::Fall,
                "Du bringst eine instabile Reihe aus dem Gleichgewicht. Die Szene endet unter Material.",
            ),
            "lost_signal" => self.trigger_consequence(
                DeathKind::Signal,
                "Du berührst den fehlenden Bildbereich. Der Frame verliert die Synchronität.",
            ),
            _ => {
                self.status = format!(
                    "Du stubst {} an. Es gibt keine verwertbare Bewegung.",
                    hotspot.name
                );
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
            "greenscreen_wall" | "floor_marks" | "gaffer_roll" | "chat_preview" => {
                self.trigger_consequence(
                    DeathKind::Fire,
                    "Du probierst Produktionsmaterial mit der Zunge, reißt reflexartig am Aufbau und löst den Sprinkleralarm aus.",
                );
            }
            "wet_note" | "flow_arrow" | "archive_hatch" => {
                self.set_flag_log(
                    "wet_note_smell_checked",
                    "Der nasse Timecode wird eindeutig dem Kanal und der Luke zugeordnet. Die Methode bleibt fragwürdig.",
                    "Der Timecode ist zugeordnet. Die Methode war unnötig direkt.",
                );
            }
            "mold_rack" | "mold_token" => {
                self.set_flag_log(
                    "mold_material_checked",
                    "Die Materialprobe ist über Oberfläche und Geschmack zugeordnet.",
                    "Die Formprobe ist zugeordnet. Das Protokoll erwähnt nicht die Methode.",
                );
            }
            "lost_signal" => {
                self.trigger_consequence(
                    DeathKind::Signal,
                    "Du prüfst einen fehlenden Bildbereich mit der Zunge. Die Regie muss den Frame technisch bergen.",
                );
            }
            _ => {
                self.status = format!("{} lässt sich hier nicht sinnvoll anzüngeln.", hotspot.name);
            }
        }
    }

    fn smell(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            return;
        };

        match id {
            "mold_rack" | "mold_token" => self.set_flag_log(
                "mold_material_checked",
                "Der Geruch bestätigt: Die helle Probe gehört zur frischen Formenreihe.",
                "Die Formprobe ist über Materialgeruch zugeordnet.",
            ),
            "archive_hatch" if self.has_item("wet_note") => self.set_flag_log(
                "wet_note_smell_checked",
                "Der Geruch an Luke und Timecode passt zusammen: gleicher nasser Papier- und Metallton.",
                "Die Archivluke gehört zum nassen Timecode.",
            ),
            "wet_note" => self.set_flag_log(
                "wet_note_smell_checked",
                "Der nasse Timecode riecht nach Kanalwasser und frischer Tinte.",
                "Der Timecode ist frisch genug, um zur Archivluke zu gehören.",
            ),
            "factory_floor" => self.status =
                "Der Boden riecht nach feuchtem Material, nicht nach einem Ausgang.".to_string(),
            "greenscreen_wall" => {
                self.status = "Der Stoff riecht nach trockenem Molton und Staub.".to_string();
            }
            "lost_signal" => self.trigger_consequence(
                DeathKind::Signal,
                "Der fehlende Bildbereich riecht nach heißem Videocodec. Danach riecht nichts mehr.",
            ),
            _ => {
                self.status = format!("{} liefert keinen nützlichen Geruch.", hotspot.name);
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
                "Laufzettel aufgehoben.",
                "Laufzettel aufgehoben: Studio, Regie und Außenreferenzen sind als Auftrag vermerkt.",
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
                "Bauchbindenkarte aufgehoben.",
                "Die freigegebene Bauchbindenkarte wurde im Copyshop abgeholt.",
            ),
            "city_reflector" => self.take_item(
                "city_reflector",
                "city_reflector_taken",
                "Lichtreflektor aufgehoben.",
                "Der Lichtreflektor wurde als Referenzmaterial für das Studio aufgenommen.",
            ),
            "wet_note" => self.take_item(
                "wet_note",
                "wet_note_taken",
                "Nassen Timecode aufgehoben.",
                "Im Kanalvideo liegt ein verwischter Timecode: Zulauf, Ablauf, Gegenlauf.",
            ),
            "mold_token" => self.take_item(
                "mold_token",
                "mold_token_taken",
                "Formprobe aufgehoben.",
                "Die Formprobe aus der Schimmelbrüder-Halle wurde gesichert.",
            ),
            "road_marker" => self.take_item(
                "road_marker",
                "road_marker_taken",
                "Routenmarker aufgehoben.",
                "Der Marker aus dem Straßenframe wurde gesichert.",
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
                self.status = format!("{} ist kein loser Gegenstand.", hotspot.name);
            }
        }
    }

    fn use_hotspot(&mut self, id: &str) {
        let Some(hotspot) = self.hotspot(id) else {
            self.status = "Dieser Punkt ist in dieser Szene nicht aktiv.".to_string();
            return;
        };

        if hotspot.kind == HotspotKind::Exit {
            self.travel(
                hotspot.id,
                &format!("Du gehst zu {}.", current_scene(hotspot.id).name),
            );
            return;
        }

        match id {
            "call_button" => self.set_flag_log(
                "nurse_called",
                "Der Rufknopf wurde benutzt. Die Pflegekraft kommt ins Zimmer.",
                "Die Pflegekraft ist informiert.",
            ),
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
            "archive_hatch" => self.try_open_archive_hatch(),
            "archive_exit_sign" | "distant_gate" => self.try_open_road_exit(),
            "song_exit_gate" => self.try_open_schimmel_exit(),
            "greenscreen_setup" => self.try_finish_loop(),
            _ if hotspot.talk_id.is_some() => self.talk(id),
            _ if hotspot.kind == HotspotKind::Pickup => {
                let target = pickup_target_name(id).unwrap_or(hotspot.name);
                self.status = format!("{target} ist ein Gegenstand. Nimm ihn mit Aufheben.");
            }
            _ => {
                self.status = format!("{} hat hier keine Benutzen-Funktion.", hotspot.name);
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
                "Die Dispo-Tafel bestätigt: Studio, Regie, Außenreferenzen.",
                "Der Tagesplan bestätigt die Reihenfolge der Aufgaben.",
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
            "graphics_terminal" => self.set_flag_log(
                "graphics_slot_checked",
                "Der Grafikplatz wartet auf die freigegebene Bauchbindenkarte.",
                "Die Regie benötigt die freigegebene Bauchbindenkarte.",
            ),
            "walkable_lane" => self.set_flag_log(
                "road_plane_checked",
                "Die Standspur ist breit genug für die Figur.",
                "Begehbare Ebene im Straßenframe bestätigt.",
            ),
            "traffic_totem" => self.set_flag_log(
                "road_order_seen",
                "Die Markierungen sortieren die Tiefe. Anstubsen bestätigt die Reihenfolge: nah, mittel, fern.",
                "Die Markierungsgruppe zeigt die Tiefenreihenfolge. Stubse sie zur Bestätigung an.",
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
            "flow_arrow" => self.set_flag_log(
                "sewer_flow_checked",
                "Der Flusspfeil markiert die Reihenfolge: Zulauf, Ablauf, Gegenlauf.",
                "Der Pfeil markiert die Reihenfolge für den Timecode.",
            ),
            "inspection_tripod" => self.set_flag_log(
                "sewer_tripod_checked",
                "Das Inspektionsstativ ist auf die Luke ausgerichtet.",
                "Das Stativ ist auf die Archivluke ausgerichtet.",
            ),
            "lost_signal" => self.set_flag_log(
                "sewer_signal_seen",
                "Im Videoframe fehlt ein Bildbereich.",
                "Fehlender Bildbereich im Kanalvideo dokumentiert.",
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
            self.status = "Hier ist niemand, der antwortet.".to_string();
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
                    "Die Bauchbindenkarte ist am Grafikplatz eingepflegt.",
                    "Bauchbinde geladen.",
                );
            }
            ("camera_one" | "greenscreen_setup", "city_reflector") => {
                self.set_flag_log(
                    "city_reflector_placed",
                    "Der Lichtreflektor wurde am Studio-Setup platziert.",
                    "Lichtreferenz platziert.",
                );
            }
            ("greenscreen_setup", "call_sheet") => {
                self.set_flag_log(
                    "call_sheet_checked",
                    "Der Laufzettel liegt am Setup.",
                    "Laufzettel am Setup geprüft.",
                );
                self.try_finish_loop();
            }
            ("archive_hatch", "wet_note") => {
                self.try_open_archive_hatch();
            }
            ("archive_exit_sign" | "distant_gate", "road_marker") => {
                self.try_open_road_exit();
            }
            ("song_exit_gate" | "patterned_belt", "mold_token") => {
                self.try_open_schimmel_exit();
            }
            ("hospital_exit", "medical_release") => {
                self.try_leave_hospital();
            }
            ("recovery_terminal", "checksum_note") => {
                self.try_close_recovery_session();
            }
            _ => {
                let item = item_meta(item_id).map(|i| i.name).unwrap_or("Das");
                self.status = format!("{} passt hier nicht.", item);
            }
        }
    }

    fn try_take_medical_release(&mut self) {
        if !self.flag("hospital_vitals_checked") {
            self.status = "Prüfe zuerst den Monitor. Ohne stabile Werte gibt es keine Entlassung."
                .to_string();
            return;
        }
        if !self.flag("nurse_clearance") {
            self.status =
                "Sprich mit der Pflegekraft. Der Bogen ist noch nicht freigegeben.".to_string();
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

    fn try_open_road_exit(&mut self) {
        if self.flag("road_video_cleared") {
            self.status = "Die Ausfahrt im Straßenframe ist freigegeben.".to_string();
            return;
        }
        if !self.flag("road_plane_checked") {
            self.status = "Prüfe zuerst die Standfläche der Figur in diesem Frame.".to_string();
            return;
        }
        if !self.has_item("road_marker") {
            self.status = "Die Ausfahrt braucht den Marker aus dem Straßenvideo.".to_string();
            return;
        }
        if !self.flag("road_order_checked") {
            self.status =
                "Der Marker allein reicht nicht. Stubse die rot-weißen Markierungen an, um die Bildtiefe zu bestätigen."
                    .to_string();
            return;
        }
        self.state.flags.insert("road_video_cleared".to_string());
        self.add_log(
            "Der Routenmarker wurde der Straßenausfahrt zugeordnet. Der Straßenframe ist als begehbarer Bereich dokumentiert.",
        );
        self.status = "Die rechte Fahrspur ist freigegeben.".to_string();
    }

    fn try_open_schimmel_exit(&mut self) {
        if self.flag("schimmel_video_cleared") {
            self.status =
                "Der Clip-Ausgang ist freigegeben. Die Formenreihe bleibt begehbar.".to_string();
            return;
        }
        if !self.flag("schimmel_floor_checked") {
            self.status =
                "Prüfe zuerst die Standfläche der Figur in dieser Fertigungshalle.".to_string();
            return;
        }
        if !self.has_item("mold_token") {
            self.status = "Dem Clip-Ausgang fehlt eine Probe aus der Formenreihe.".to_string();
            return;
        }
        if !self.flag("mold_material_checked") {
            self.status =
                "Die Probe ist noch nicht zugeordnet. Rieche an der Formenreihe, bevor du sie am Ausgang benutzt."
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
            "Die Formprobe wurde dem Clip-Ausgang zugeordnet. Der Schimmelbrüder-Frame ist freigegeben.",
        );
        self.status = "Formprobe zugeordnet. Der Clip-Ausgang ist freigegeben.".to_string();
    }

    fn try_open_archive_hatch(&mut self) {
        if self.flag("sewer_video_cleared") {
            self.status = "Die Archivluke ist freigegeben.".to_string();
            return;
        }
        if !self.has_item("wet_note") {
            self.status =
                "Die Luke will einen Timecode. Im Tunnel muss noch ein Hinweis liegen.".to_string();
            return;
        }
        if !self.flag("sewer_flow_checked") {
            self.status =
                "Der Timecode allein reicht nicht. Die Pfeilrichtung bestimmt die Reihenfolge."
                    .to_string();
            return;
        }
        if !self.flag("wet_note_smell_checked") {
            self.status =
                "Der Timecode ist noch nicht der Luke zugeordnet. Rieche an Luke oder Hinweis."
                    .to_string();
            return;
        }
        self.state.flags.insert("sewer_video_cleared".to_string());
        self.add_log(
            "Der Timecode wurde der Archivluke zugeordnet. Der nächste Videoraum ist erreichbar.",
        );
        self.status = "Die Archivluke ist freigegeben.".to_string();
    }

    fn try_finish_loop(&mut self) {
        if !self.has_item("call_sheet") {
            self.status = "Der Laufzettel fehlt. Hol ihn am Dispo-Board.".to_string();
            return;
        }
        if !self.flag("greenscreen_marked") {
            self.status =
                "Die Bodenmarken fehlen noch. Nutze das Gaffer-Tape im Studio.".to_string();
            return;
        }
        if !self.flag("route_labeled") {
            self.status =
                "Der Signalweg ist noch nicht beschriftet. Das SDI-Label gehört an die Route."
                    .to_string();
            return;
        }
        if !self.flag("graphic_loaded") {
            self.status = "Die Bauchbinde fehlt am Grafikplatz.".to_string();
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
        self.state.flags.insert("rehearsal_ready".to_string());
        self.add_log("Der Probelauf ist vollständig vorbereitet: Studio, Regie und Außenreferenzen sind abgeglichen.");
        self.status = "Die Regie bestätigt den Probelauf.".to_string();
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

        self.draw_player();
    }

    fn draw_player(&self) {
        let token = self.player_pos;
        let frame = if self.walk_target.is_some() {
            ((get_time() * 8.0) as usize % 4) as f32
        } else {
            0.0
        };

        if let Some(texture) = &self.player_texture {
            let frame_w = texture.width() / 4.0;
            let frame_h = texture.height() / 4.0;
            let dest = vec2(PLAYER_DRAW_W, PLAYER_DRAW_H);
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

        let player_rect = Rect::new(
            token.x - 24.0,
            token.y - 96.0 * PLAYER_FOOT_ANCHOR_Y,
            48.0,
            96.0,
        );
        draw_placeholder_sprite(player_rect, "FIGUR", PlaceholderCategory::Character, false);
    }

    fn draw_scene_background(&self, scene: &SceneMeta, rect: Rect) {
        if let Some(texture) = self.scene_textures.get(scene.id) {
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
        centered_text("PROBELAUF", 640.0, 220.0, 18, amber());
        centered_text("Probelauf freigegeben", 640.0, 270.0, 34, bone());
        draw_text_wrapped(
            "Signalweg, Bauchbinde, Bodenmarken, Zeitreferenz und Lichtreferenz sind vollständig. Die Regie kann den Greenscreen-Probelauf starten.",
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
            "print_shop" => !self.has_item("lower_third_card"),
            "city_reflector" => !self.has_item("city_reflector"),
            "wet_note" => !self.has_item("wet_note"),
            "road_marker" => !self.has_item("road_marker"),
            "mold_token" => !self.has_item("mold_token"),
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
        self.status = "Probe neu gestartet. Der Laufzettel liegt wieder am Anfang.".to_string();
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
        "dispo_board" => Some("Laufzettel"),
        "gaffer_roll" => Some("Gaffer-Tape"),
        "sdi_label_printer" => Some("SDI-Label"),
        "print_shop" => Some("Bauchbindenkarte"),
        "city_reflector" => Some("Lichtreflektor"),
        "wet_note" => Some("Nasser Timecode"),
        "mold_token" => Some("Formprobe"),
        "road_marker" => Some("Routenmarker"),
        "discharge_clipboard" => Some("Entlassungsbogen"),
        "checksum_printout" => Some("Prüfsummenzettel"),
        _ => None,
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
const NATIVE_SAVE_FILE: &str = "klixx.local-save.json";

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
        "wet_note" => (2.0, 1.0),
        "road_marker" => (3.0, 0.0),
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
    draw_text_ex("PROBENAKTE", 936.0, 628.0, text_params(14, ochre()));
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
    if game.state.scene == "video_kliemannsland_road" {
        return match line {
            0 if !game.flag("road_plane_checked") => "Ansehen: Standspur",
            0 => "Begehbare Ebene gefunden",
            1 if !game.has_item("road_marker") => "Aufheben: Routenmarker",
            1 if !game.flag("road_order_checked") => "Anstubsen: Markierungen",
            1 => "Tiefe gelesen",
            2 if !game.flag("road_video_cleared") => "Benutzen: Marker an Ausfahrt",
            _ => "Strassenvideo geloest",
        };
    }
    if game.state.scene == "video_sewer_archive" {
        return match line {
            0 if !game.has_item("wet_note") => "Aufheben: Nasser Timecode",
            0 => "Timecode im Inventar",
            1 if !game.flag("sewer_flow_checked") => "Ansehen: Flusspfeil",
            1 if !game.flag("wet_note_smell_checked") => "Riechen: Luke/Timecode",
            1 => "Luke und Timecode passen",
            2 if !game.flag("sewer_video_cleared") => "Timecode an Luke nutzen",
            _ => "Kanalvideo geloest",
        };
    }
    if game.state.scene == "video_schimmelbrueder" {
        return match line {
            0 if !game.flag("schimmel_floor_checked") => "Ansehen: Hallenboden",
            0 => "Standflaeche gefunden",
            1 if !game.has_item("mold_token") => "Aufheben: Formprobe",
            1 if !game.flag("mold_material_checked") => "Riechen: Formenreihe",
            1 if !game.flag("mold_pattern_checked") => "Musterbahn lesen",
            1 => "Formenfolge dokumentiert",
            2 if !game.flag("schimmel_video_cleared") => "Benutzen: Probe am Ausgang",
            _ => "Schimmelvideo geloest",
        };
    }
    match line {
        0 if !game.has_item("call_sheet") => "Aufheben: Laufzettel",
        0 if !game.flag("greenscreen_marked") => "Bodenmarken mit Tape setzen",
        0 => "Greenscreen markiert",
        1 if !game.has_item("sdi_label") => "Aufheben: SDI-Label",
        1 if !game.flag("route_labeled") => "Signalweg beschriften",
        1 if !game.has_item("lower_third_card") => "Aufheben: Bauchbindenkarte",
        1 if !game.flag("graphic_loaded") => "Bauchbinde in Regie laden",
        1 => "Regie vorbereitet",
        2 if !game.flag("station_timed") => "Zeitreferenz am Bahnhof prüfen",
        2 if !game.flag("city_light_checked") => "Lichtreferenz am Wasserturm prüfen",
        2 if !game.has_item("city_reflector") => "Aufheben: Lichtreflektor",
        2 if !game.flag("city_reflector_placed") => "Lichtreferenz im Studio setzen",
        _ => "Benutzen: Probelauf starten",
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
        game.pick_up_hotspot("city_reflector");
        assert!(game.has_item("city_reflector"));

        game.travel("karoviertel", "test");
        game.pick_up_hotspot("print_shop");
        assert!(game.has_item("lower_third_card"));

        game.travel("control_room", "test");
        game.handle_item_use("graphics_terminal", "lower_third_card");
        assert!(game.flag("graphic_loaded"));

        game.travel("greenscreen_studio", "test");
        game.handle_item_use("floor_marks", "gaffer_tape");
        assert!(game.flag("greenscreen_marked"));
        game.handle_item_use("greenscreen_setup", "sdi_label");
        assert!(game.flag("route_labeled"));
        game.handle_item_use("greenscreen_setup", "city_reflector");
        assert!(game.flag("city_reflector_placed"));
        game.use_hotspot("greenscreen_setup");

        assert!(game.state.complete);
        assert!(game.flag("rehearsal_ready"));
        assert!(matches!(game.modal, Modal::Milestone));

        clear_save();
    }

    #[test]
    fn rehearsal_requires_city_checks_and_setup_items() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.use_hotspot("greenscreen_setup");
        assert!(matches!(game.modal, Modal::None));
        assert!(game.status.contains("Laufzettel"));

        game.add_item("call_sheet");
        game.state.flags.insert("greenscreen_marked".to_string());
        game.state.flags.insert("route_labeled".to_string());
        game.state.flags.insert("graphic_loaded".to_string());
        game.state.flags.insert("station_timed".to_string());
        game.state.flags.insert("city_light_checked".to_string());

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
        assert!(response.text.contains("Laufzettel"));
        assert!(!game.status.contains("Laufzettel"));

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
        game.use_hotspot("call_button");
        assert!(game.flag("nurse_called"));
        game.state.flags.insert("nurse_clearance".to_string());
        game.pick_up_hotspot("discharge_clipboard");
        assert!(game.has_item("medical_release"));

        game.state.verb = Verb::Smell;
        game.handle_hotspot("hospital_exit");
        assert_eq!(game.state.scene, "office_hall");

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
    fn walkable_video_room_can_be_solved() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("control_room", "test");
        game.use_hotspot("video_kliemannsland_road");
        assert_eq!(game.state.scene, "video_kliemannsland_road");
        assert!(game.hotspot("road_marker").is_some());

        game.use_hotspot("archive_exit_sign");
        assert!(!game.flag("road_video_cleared"));
        assert!(game.status.contains("Standfläche"));

        game.inspect("walkable_lane");
        assert!(game.flag("road_plane_checked"));

        game.pick_up_hotspot("road_marker");
        assert!(game.has_item("road_marker"));
        assert!(game
            .hotspots()
            .iter()
            .all(|hotspot| hotspot.id != "road_marker"));

        game.handle_item_use("archive_exit_sign", "road_marker");
        assert!(!game.flag("road_video_cleared"));
        assert!(game.status.contains("rot-weißen"));

        game.inspect("traffic_totem");
        assert!(game.flag("road_order_seen"));
        game.poke("traffic_totem");
        assert!(game.flag("road_order_checked"));

        game.handle_item_use("archive_exit_sign", "road_marker");
        assert!(game.flag("road_video_cleared"));
        assert!(game.status.contains("Fahrspur"));

        clear_save();
    }

    #[test]
    fn schimmelbrueder_video_room_can_be_solved() {
        clear_save();
        let mut game = Game::new(GameState::default());

        game.travel("control_room", "test");
        game.use_hotspot("video_schimmelbrueder");
        assert_eq!(game.state.scene, "video_schimmelbrueder");
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
        assert!(game.status.contains("Rieche"));

        game.smell("mold_rack");
        assert!(game.flag("mold_material_checked"));
        game.inspect("patterned_belt");
        assert!(game.flag("mold_pattern_checked"));

        game.handle_item_use("song_exit_gate", "mold_token");
        assert!(game.flag("schimmel_video_cleared"));
        assert!(game.status.contains("Clip-Ausgang"));

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
}
