use std::collections::{HashMap, hash_map::Entry};

#[derive(Debug, PartialEq, Eq)]
pub enum Rebind{
    ButtonToButton(usize, usize),
    AxisToAxis(usize, usize),
    HatToHat(usize, usize),
}

pub struct RebindProcessor{
    rebinds: HashMap<(String, String), Vec<Rebind>>, // <(from, to), rebinds>
}

impl RebindProcessor{
    pub fn new() -> Self{
        let rebinds = HashMap::new();
        Self { rebinds }
    }

    pub fn add_rebind(&mut self, from: String, to: String, rebind: Rebind){
        let rebinds = match self.rebinds.entry((from, to)) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Vec::new()),
        };
        rebinds.push(rebind);
    }

    pub fn remove_rebind(&mut self, from: String, to: String, rebind: Rebind){
        let Some(rebinds) = self.rebinds.get_mut(&(from, to)) else{
            return;
        };
        rebinds.retain(|reb|{
            *reb != rebind
        });
    }

    pub fn clear_rebinds_from(&mut self, clear_from: String){
        self.rebinds.retain(|(from, _to), _rebinds|{
            *from != clear_from
        });
    }

    pub fn clear_rebinds_to(&mut self, clear_to: String){
        self.rebinds.retain(|(_from, to), _rebinds|{
            *to != clear_to
        });
    }

    pub fn clear_all(&mut self){
        self.rebinds.clear();
    }
}