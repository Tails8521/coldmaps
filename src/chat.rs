use coldmaps::heatmap_analyser::HeatmapAnalysis;

pub fn format_chat_messages(analysis: &HeatmapAnalysis) -> Vec<String> {
    let interval_per_tick = analysis.interval_per_tick;
    analysis
        .chat
        .iter()
        .filter_map(|message| {
            let total_seconds = (message.tick as f32 * interval_per_tick) as u32;
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            match message.kind {
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::ChatAll => Some(format!("[{:02}:{:02}] {}: {}", minutes, seconds, message.from, message.text,)),
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::ChatTeam => {
                    Some(format!("[{:02}:{:02}] (TEAM) {}: {}", minutes, seconds, message.from, message.text,))
                }
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::ChatAllDead => {
                    Some(format!("[{:02}:{:02}] *DEAD* {}: {}", minutes, seconds, message.from, message.text,))
                }
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::ChatTeamDead => {
                    Some(format!("[{:02}:{:02}] *DEAD*(TEAM) {}: {}", minutes, seconds, message.from, message.text,))
                }
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::ChatAllSpec => {
                    Some(format!("[{:02}:{:02}] *SPEC* {}: {}", minutes, seconds, message.from, message.text,))
                }
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::NameChange => None,
                tf_demo_parser::demo::message::usermessage::ChatMessageKind::Empty => None,
            }
        })
        .collect()
}
