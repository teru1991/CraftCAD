pub fn merge_selection(current: &mut Vec<String>, incoming: &[String], additive: bool) {
    if !additive {
        current.clear();
    }
    for id in incoming {
        if !current.contains(id) {
            current.push(id.clone());
        }
    }
    current.sort();
}
