//! Basketball vocabulary as data: events (S1 §4), learning targets (S6 §3), reads (S6 §5),
//! format choices (S4) and play actions (S7).

/// `(id, label, live)` — `live` = a coach can tap it mid-play (S1 §4–5).
pub const EVENTS: &[(&str, &str, bool)] = &[
    ("make", "Basket made", true),
    ("miss", "Missed shot", false),
    ("shot", "Shot", false),
    ("off-reb", "Offensive rebound", true),
    ("def-reb", "Defensive rebound", false),
    ("loose-ball", "Loose ball", true),
    ("paint-touch", "Paint touch", true),
    ("pass", "Pass", false),
    ("catch", "Catch", false),
    ("dribble", "Dribble", false),
    ("drive", "Drive", true),
    ("screen", "Screen", false),
    ("cut", "Cut", false),
    ("closeout", "Closeout", true),
    ("extra-pass", "Extra pass", false),
    ("assist", "Assist", true),
    ("stop", "Stop", true),
    ("deflection", "Deflection", true),
    ("steal", "Steal", true),
    ("block", "Blocked shot", true),
    ("charge", "Charge taken", true),
    ("contest", "Contest", false),
    ("turnover", "Turnover", true),
    ("violation", "Violation", true),
    ("foul", "Foul", true),
    ("talk", "Talk", true),
    ("sprint-back", "Sprint back", true),
    ("capture", "Capture (players found something)", true),
    ("zone.enter", "Enters a zone", false),
    ("zone.exit", "Leaves a zone", false),
    ("role.enter", "Player joins", false),
    ("role.leave", "Player leaves", false),
    ("possession.start", "Possession starts", false),
    ("possession.end", "Possession ends", false),
    ("phase.change", "Phase changes", false),
    ("dial.set", "Dial set", false),
    ("constraint.on", "Rule switched on", false),
    ("constraint.off", "Rule switched off", false),
    ("clock.expired", "Clock runs out", false),
    ("clock.warning", "Clock warning", false),
];

pub fn event_label(id: &str) -> &str {
    EVENTS.iter().find(|e| e.0 == id).map(|e| e.1).unwrap_or(id)
}

/// Named regions (S3 §5, §7). `(id, label)`.
pub const NAMED_REGIONS: &[(&str, &str)] = &[
    ("paint", "Paint"),
    ("restricted", "Restricted arc"),
    ("corner-L", "Left corner"),
    ("corner-R", "Right corner"),
    ("wing-L", "Left wing"),
    ("wing-R", "Right wing"),
    ("top", "Top"),
    ("above-break", "Above the break"),
    ("elbow-L", "Left elbow"),
    ("elbow-R", "Right elbow"),
    ("block-L", "Left block"),
    ("block-R", "Right block"),
    ("short-corner-L", "Left short corner"),
    ("short-corner-R", "Right short corner"),
    ("dunker-L", "Left dunker spot"),
    ("dunker-R", "Right dunker spot"),
    ("mid-range", "Mid-range"),
    ("inside-arc", "Inside the arc"),
    ("beyond-arc", "Beyond the arc"),
    ("backcourt", "Backcourt"),
];

/// Court regions a game can be played on (S3 §5). `(id, label)`.
pub const COURT_REGIONS: &[(&str, &str)] = &[
    ("half", "Half court"),
    ("full", "Full court"),
    ("three-quarter", "Three-quarter court"),
    ("cross-court", "Cross-court (side basket)"),
    ("inside-arc", "Inside the arc"),
];

/// Other place names formats use (start spots).
pub const PLACES: &[&str] = &["half-court-line", "baseline-out"];

pub fn region_label(id: &str) -> &str {
    NAMED_REGIONS
        .iter()
        .chain(COURT_REGIONS.iter())
        .find(|r| r.0 == id)
        .map(|r| r.1)
        .unwrap_or(id)
}

/// The learning-target tree (S6 §3). `(id, label)`.
pub const TARGETS: &[(&str, &str)] = &[
    ("off", "Offense"),
    ("off.create", "Create an advantage"),
    ("off.create.beat-1v1", "Beat your player"),
    ("off.create.ball-screen", "Use a ball screen"),
    ("off.create.off-ball-screen", "Use / set an off-ball screen"),
    ("off.create.cut", "Cut"),
    ("off.create.post-seal", "Post seal"),
    ("off.create.transition-push", "Push in transition"),
    ("off.extend", "Extend the advantage"),
    ("off.extend.drive-and-kick", "Drive and kick"),
    ("off.extend.attack-closeout", "Attack the closeout"),
    ("off.extend.extra-pass", "Extra pass"),
    ("off.extend.short-roll", "Short roll"),
    ("off.extend.skip-pass", "Skip pass"),
    ("off.finish", "Finish"),
    (
        "off.finish.rim-vs-contest",
        "Finish at the rim vs. a contest",
    ),
    ("off.finish.shot-selection", "Shot selection"),
    ("off.finish.off-reb", "Offensive rebound"),
    ("off.shape", "Shape"),
    ("off.shape.spacing", "Spacing"),
    ("off.shape.timing", "Timing"),
    ("off.shape.relocation", "Relocation"),
    ("off.shape.balance", "Floor balance"),
    ("def", "Defense"),
    ("def.deny", "Deny the advantage"),
    ("def.deny.contain", "On-ball containment"),
    ("def.deny.screen-coverage", "Screen coverage"),
    ("def.deny.gap", "Deny / gap position"),
    ("def.scramble", "Scramble"),
    ("def.scramble.help", "Help"),
    ("def.scramble.rotate", "Rotate"),
    ("def.scramble.closeout", "Close out"),
    ("def.scramble.x-out", "X-out"),
    ("def.scramble.talk", "Talk"),
    ("def.end", "End the possession"),
    ("def.end.contest", "Contest without fouling"),
    ("def.end.box-out", "Box out"),
    ("def.end.secure", "Secure the rebound"),
    ("trans.off", "Transition offense"),
    ("trans.off.outlet", "Outlet"),
    ("trans.off.fill-lanes", "Fill lanes"),
    ("trans.off.early-advantage", "Early advantage"),
    ("trans.def", "Transition defense"),
    ("trans.def.get-back", "Get back"),
    ("trans.def.stop-ball", "Stop the ball"),
    ("trans.def.match-up", "Match up"),
    ("char.humble", "Humble — extra pass, screening for others"),
    ("char.hungry", "Hungry — loose balls, offensive rebounds"),
    ("char.effort", "Effort — sprint back, closeouts, help"),
    (
        "char.attitude",
        "Attitude — next-play response, encouragement, talk",
    ),
];

pub fn target_label(id: &str) -> &str {
    TARGETS
        .iter()
        .find(|t| t.0 == id)
        .map(|t| t.1)
        .unwrap_or(id)
}

/// Top-level label for a target id ("Offense · Extend the advantage").
pub fn target_path_label(id: &str) -> String {
    let parts: Vec<&str> = id.split('.').collect();
    (1..=parts.len())
        .map(|n| parts[..n].join("."))
        .filter(|id| TARGETS.iter().any(|t| t.0 == id))
        .map(|id| target_label(&id).to_string())
        .collect::<Vec<_>>()
        .join(" · ")
}

pub struct ReadDef {
    pub id: &'static str,
    pub target: &'static str,
    pub cue: &'static str,
    /// Whose behaviour is the cue (S6 §4).
    pub source: &'static str,
    pub options: &'static [(&'static str, bool)],
}

/// Starter reads (S6 §5).
pub const READS: &[ReadDef] = &[
    ReadDef {
        id: "read.closeout.high-fast",
        target: "off.extend.attack-closeout",
        cue: "The closeout is high and fast",
        source: "closing defender",
        options: &[("Drive past the top foot", true), ("Shoot", false)],
    },
    ReadDef {
        id: "read.closeout.short",
        target: "off.extend.attack-closeout",
        cue: "The closeout is short or late",
        source: "closing defender",
        options: &[("Shoot", true), ("Drive", false)],
    },
    ReadDef {
        id: "read.help.steps-up",
        target: "off.extend.drive-and-kick",
        cue: "Help steps up to the drive",
        source: "help defender",
        options: &[("Kick to the helper's player", true), ("Finish", false)],
    },
    ReadDef {
        id: "read.help.stays",
        target: "off.extend.drive-and-kick",
        cue: "Help stays home",
        source: "help defender",
        options: &[("Finish", true), ("Kick", false)],
    },
    ReadDef {
        id: "read.ballscreen.drop",
        target: "off.create.ball-screen",
        cue: "The screener's defender drops",
        source: "screener's defender",
        options: &[("Pull-up or pocket pass", true)],
    },
    ReadDef {
        id: "read.ballscreen.hedge",
        target: "off.create.ball-screen",
        cue: "Hedge or show",
        source: "screener's defender",
        options: &[("Hit the popper or the short roll", true)],
    },
    ReadDef {
        id: "read.ballscreen.switch",
        target: "off.create.ball-screen",
        cue: "They switch",
        source: "both defenders",
        options: &[("Attack the big or seal the small", true)],
    },
    ReadDef {
        id: "read.ballscreen.help-on-roll",
        target: "off.create.ball-screen",
        cue: "Weak-side help on the roller",
        source: "help defender",
        options: &[("Lift and kick to the helper's player", true)],
    },
    ReadDef {
        id: "read.denial.overplay",
        target: "off.create.cut",
        cue: "The wing is denied",
        source: "wing defender",
        options: &[("Back cut", true)],
    },
    ReadDef {
        id: "read.2v1.commit",
        target: "trans.off.early-advantage",
        cue: "The defender commits to the ball",
        source: "lone defender",
        options: &[("Pass", true)],
    },
    ReadDef {
        id: "read.2v1.stays-home",
        target: "trans.off.early-advantage",
        cue: "The defender stays with the receiver",
        source: "lone defender",
        options: &[("Finish", true)],
    },
];

pub fn read(id: &str) -> Option<&'static ReadDef> {
    READS.iter().find(|r| r.id == id)
}

pub fn read_label(id: &str) -> String {
    read(id)
        .map(|r| r.cue.to_string())
        .unwrap_or_else(|| id.to_string())
}

/// Format choices (S4). `(id, label)`.
pub const START_TYPES: &[(&str, &str)] = &[
    ("check-ball", "Check ball"),
    ("coach-pass", "Coach passes in"),
    ("closeout", "Defense passes out and closes out"),
    ("rebound", "Coach shoots, play the rebound"),
    ("transition", "Outlet and go"),
    ("inbound", "Inbound"),
    ("dribble-at", "Attack from a spot"),
];

pub const RESTARTS: &[(&str, &str)] = &[
    ("switch", "Switch: other team's ball"),
    ("keep", "Keep: make it, take it"),
    ("rotate", "Rotate teams"),
    ("check", "Check it up"),
    ("phase-next", "Next phase"),
];

pub const ROTATIONS: &[(&str, &str)] = &[
    ("none", "Same two teams"),
    ("winners-stay", "Winners stay"),
    ("losers-stay", "Losers stay"),
    ("three-team", "Three teams (O → D → off)"),
    ("round-robin", "Round robin"),
    ("king", "King of the court"),
    ("stations", "Stations"),
    ("continuous", "Continuous"),
];

pub const END_TYPES: &[(&str, &str)] = &[
    ("score", "First to a score"),
    ("time", "Timed game"),
    ("possessions", "Set number of possessions"),
    ("block", "Play until the block ends"),
];

pub fn label_of<'a>(table: &'a [(&'a str, &'a str)], id: &'a str) -> &'a str {
    table.iter().find(|t| t.0 == id).map(|t| t.1).unwrap_or(id)
}

/// Play actions (S7 §2). `(id, label)`.
pub const ACTIONS: &[(&str, &str)] = &[
    ("pass", "Pass"),
    ("dribble", "Dribble"),
    ("cut", "Cut"),
    ("screen", "Screen"),
    ("handoff", "Handoff"),
    ("shot", "Shot"),
];

/// Group modes for practice blocks (S8 §5).
pub const GROUP_MODES: &[(&str, &str)] = &[
    ("random", "Random"),
    ("balanced", "Balanced"),
    ("ability", "By ability"),
    ("keep", "Same as last block"),
    ("explicit", "Set teams"),
];

/// Coach-language body-part words the cue linter flags (R14).
pub const BODY_WORDS: &[&str] = &[
    "elbow",
    "wrist",
    "knee",
    "knees",
    "hips",
    "hip",
    "shoulder",
    "shoulders",
    "fingers",
    "finger",
    "feet",
    "foot",
    "ankle",
    "arm",
    "arms",
    "hand",
    "hands",
    "head",
];

/// R14: words in a cue that point at the body instead of the environment.
pub fn body_words(cue: &str) -> Vec<&'static str> {
    let lower = cue.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    BODY_WORDS
        .iter()
        .copied()
        .filter(|b| words.contains(b))
        .collect()
}
