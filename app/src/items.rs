use crate::world::{FLOOR_Y, STOREY};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Crowbar,
    Umbrella,
    Box,
    BlueKey,
    GreenKey,
    RedKey,
}
impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Crowbar => "Crowbar",
            Self::Umbrella => "Umbrella",
            Self::Box => "Box",
            Self::BlueKey => "Blue key",
            Self::GreenKey => "Green key",
            Self::RedKey => "Red key",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MysteryDoor {
    Archive,
    Greenhouse,
    Tower,
}
impl MysteryDoor {
    pub const ALL: [Self; 3] = [Self::Archive, Self::Greenhouse, Self::Tower];
    pub fn center(self) -> Vec3 {
        match self {
            Self::Archive => Vec3::new(-8., FLOOR_Y + STOREY * 2. + 1.3, -4.5),
            Self::Greenhouse => Vec3::new(-8., FLOOR_Y + STOREY * 4. + 1.3, -4.5),
            Self::Tower => Vec3::new(2., FLOOR_Y + STOREY * 5. + 1.3, -4.),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Archive => "Blue archive",
            Self::Greenhouse => "Green greenhouse",
            Self::Tower => "Red tower",
        }
    }
    pub fn is_open(self, p: &PuzzleProgress) -> bool {
        match self {
            Self::Archive => p.archive_open,
            Self::Greenhouse => p.greenhouse_open,
            Self::Tower => p.tower_open,
        }
    }
    pub fn unlock(
        self,
        p: &mut PuzzleProgress,
        i: &Inventory,
    ) -> Result<&'static str, &'static str> {
        if self.is_open(p) {
            return Ok("The door is already open.");
        }
        match self {
            Self::Archive if i.blue_key => {
                p.archive_open = true;
                Ok("The archive is open. A green key is hidden among the house records.")
            }
            Self::Greenhouse if i.green_key => {
                p.greenhouse_open = true;
                Ok("The greenhouse is open. Find the red key for the tower.")
            }
            Self::Tower if i.red_key => {
                p.tower_open = true;
                Ok("The tower is open. Climb the ladder to the windmill machinery.")
            }
            Self::Archive => {
                Err("You need the blue key. Check the telephone table in the living room.")
            }
            Self::Greenhouse => Err("You need the green key from the archive on floor 3."),
            Self::Tower => Err("You need the red key from the greenhouse on floor 5."),
        }
    }
}
#[derive(Component)]
pub struct Pickup {
    pub kind: ItemKind,
    pub home: Vec3,
    pub velocity: Vec3,
    pub held: bool,
}
#[derive(Resource, Default)]
pub struct Inventory {
    pub crowbar: bool,
    pub keycard: bool,
    pub umbrella: bool,
    pub blue_key: bool,
    pub green_key: bool,
    pub red_key: bool,
    pub carrying: Option<Entity>,
}
#[derive(Resource, Default, Debug)]
pub struct PuzzleProgress {
    pub archive_open: bool,
    pub greenhouse_open: bool,
    pub tower_open: bool,
    pub windmill_stopped: bool,
    pub power_off: bool,
    pub train_running: bool,
    pub boards_removed: bool,
    pub basement_open: bool,
}
impl PuzzleProgress {
    pub fn objective(&self, i: &Inventory) -> &'static str {
        if self.basement_open {
            "The secret beneath the house is waiting."
        } else if !i.blue_key {
            "Find the blue key in the living room."
        } else if !i.crowbar {
            "Find the crowbar in the workshop on floor 2."
        } else if !i.green_key {
            "Use the blue key at the archive on floor 3. Find the green key."
        } else if !i.red_key {
            "Open the greenhouse on floor 5. Find the red key."
        } else if !self.tower_open {
            "Use the red key at the tower in the roof garden."
        } else if !self.windmill_stopped {
            "Climb the tower ladder. Stop the windmill machinery at the top."
        } else if !self.power_off {
            "Switch off the red generator in the greenhouse on floor 5."
        } else if !i.keycard {
            "Return to the kitchen. Take the keycard from the freezer."
        } else if !self.boards_removed {
            "Use the crowbar to remove the basement door boards."
        } else {
            "Unlock the basement door with the keycard."
        }
    }
    pub fn open_basement(&mut self, i: &Inventory) -> Result<bool, &'static str> {
        if !self.boards_removed {
            if !i.crowbar {
                return Err("You need a crowbar. Search the workshop on floor 2.");
            }
            self.boards_removed = true;
            return Ok(false);
        }
        if !i.keycard {
            return Err("You need the keycard from the kitchen freezer.");
        }
        self.basement_open = true;
        Ok(true)
    }
    pub fn disable_generator(&mut self) -> Result<(), &'static str> {
        if !self.windmill_stopped {
            return Err(
                "The windmill is still supplying power. Climb to the top of the red tower.",
            );
        }
        self.power_off = true;
        Ok(())
    }
    pub fn take_keycard(&self, i: &mut Inventory) -> Result<(), &'static str> {
        if !self.power_off {
            return Err("The freezer is iced shut. Cut the generator power upstairs.");
        }
        i.keycard = true;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basement_requires_complete_puzzle_chain() {
        let mut p = PuzzleProgress::default();
        let mut i = Inventory::default();
        assert!(p.open_basement(&i).is_err());
        assert!(p.disable_generator().is_err());
        assert!(p.take_keycard(&mut i).is_err());
        i.crowbar = true;
        assert_eq!(p.open_basement(&i), Ok(false));
        assert!(p.open_basement(&i).is_err());
        p.windmill_stopped = true;
        p.disable_generator().unwrap();
        p.take_keycard(&mut i).unwrap();
        assert_eq!(p.open_basement(&i), Ok(true));
        assert!(p.basement_open);
    }
    #[test]
    fn every_mystery_door_checks_its_own_key_and_preserves_other_locks() {
        let mut p = PuzzleProgress::default();
        let mut i = Inventory::default();
        for door in MysteryDoor::ALL {
            assert!(door.unlock(&mut p, &i).is_err());
        }
        i.blue_key = true;
        MysteryDoor::Archive.unlock(&mut p, &i).unwrap();
        assert!(p.archive_open && !p.greenhouse_open && !p.tower_open);
        assert!(MysteryDoor::Tower.unlock(&mut p, &i).is_err());
        i.green_key = true;
        MysteryDoor::Greenhouse.unlock(&mut p, &i).unwrap();
        i.red_key = true;
        MysteryDoor::Tower.unlock(&mut p, &i).unwrap();
        assert!(MysteryDoor::ALL.into_iter().all(|d| d.is_open(&p)));
    }
}
