counted_enum! {
    /// Card types (rule 205.2a)
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum CardType {
        Artifact,
        Battle,
        Conspiracy,
        Creature,
        Dungeon,
        Enchantment,
        Instant,
        Kindred,
        Land,
        Phenomenon,
        Plane,
        Planeswalker,
        Scheme,
        Sorcery,
        Vanguard,
    }
}

impl CardType {
    /// This type's slot in a table with a slot per type (CR 300.1), of `COUNT`
    /// slots. Exhaustive, so a new type fails to compile here until it has one.
    pub const fn slot(self) -> usize {
        match self {
            CardType::Artifact => 0,
            CardType::Battle => 1,
            CardType::Conspiracy => 2,
            CardType::Creature => 3,
            CardType::Dungeon => 4,
            CardType::Enchantment => 5,
            CardType::Instant => 6,
            CardType::Kindred => 7,
            CardType::Land => 8,
            CardType::Phenomenon => 9,
            CardType::Plane => 10,
            CardType::Planeswalker => 11,
            CardType::Scheme => 12,
            CardType::Sorcery => 13,
            CardType::Vanguard => 14,
        }
    }

    /// Whether this card type represents a permanent type (rule 110.4)
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            CardType::Artifact
                | CardType::Battle
                | CardType::Creature
                | CardType::Enchantment
                | CardType::Land
                | CardType::Planeswalker
        )
    }
}

/// A set of card types (CR 205.2a), one bit per [`CardType::slot`]: what a
/// card and a frame carry. A type test is a mask rather than a SipHash, and
/// the set is `Copy`, so cloning a frame allocates nothing for it.
///
/// A hand-rolled bitmask like [`crate::types::zones::ZoneSet`], with
/// `HashSet`'s method signatures, so a caller reads as it did when this was
/// one. Iteration is in slot order.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct CardTypes(u16);

const _: () = assert!(CardType::COUNT <= u16::BITS as usize);

/// Every type, in slot order: what [`CardTypes::iter`] hands out references
/// into, so they outlive the set.
static EVERY_CARD_TYPE: [CardType; CardType::COUNT] = CardType::ALL;

impl CardTypes {
    pub const fn new() -> CardTypes {
        CardTypes(0)
    }

    const fn bit(card_type: CardType) -> u16 {
        1 << card_type.slot()
    }

    pub fn contains(&self, card_type: &CardType) -> bool {
        self.0 & CardTypes::bit(*card_type) != 0
    }

    /// Adds `card_type`, answering whether it was absent.
    pub fn insert(&mut self, card_type: CardType) -> bool {
        let absent = !self.contains(&card_type);
        self.0 |= CardTypes::bit(card_type);
        absent
    }

    /// Removes `card_type`, answering whether it was present.
    pub fn remove(&mut self, card_type: &CardType) -> bool {
        let present = self.contains(card_type);
        self.0 &= !CardTypes::bit(*card_type);
        present
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&CardType) -> bool) {
        for card_type in (CardTypesIter { bits: self.0, next: 0 }) {
            if !keep(card_type) {
                self.remove(card_type);
            }
        }
    }

    pub fn iter(&self) -> CardTypesIter {
        CardTypesIter { bits: self.0, next: 0 }
    }
}

/// [`CardTypes::iter`]: each type in the set, in slot order.
pub struct CardTypesIter {
    bits: u16,
    next: usize,
}

impl Iterator for CardTypesIter {
    type Item = &'static CardType;

    fn next(&mut self) -> Option<&'static CardType> {
        while let Some(card_type) = EVERY_CARD_TYPE.get(self.next) {
            self.next += 1;
            if self.bits & CardTypes::bit(*card_type) != 0 {
                return Some(card_type);
            }
        }
        None
    }
}

impl IntoIterator for &CardTypes {
    type Item = &'static CardType;
    type IntoIter = CardTypesIter;

    fn into_iter(self) -> CardTypesIter {
        self.iter()
    }
}

impl FromIterator<CardType> for CardTypes {
    fn from_iter<I: IntoIterator<Item = CardType>>(types: I) -> CardTypes {
        let mut set = CardTypes::new();
        set.extend(types);
        set
    }
}

impl Extend<CardType> for CardTypes {
    fn extend<I: IntoIterator<Item = CardType>>(&mut self, types: I) {
        for card_type in types {
            self.insert(card_type);
        }
    }
}

impl<'a> Extend<&'a CardType> for CardTypes {
    fn extend<I: IntoIterator<Item = &'a CardType>>(&mut self, types: I) {
        self.extend(types.into_iter().copied());
    }
}

impl<const N: usize> From<[CardType; N]> for CardTypes {
    fn from(types: [CardType; N]) -> CardTypes {
        types.into_iter().collect()
    }
}

impl std::fmt::Debug for CardTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// Supertypes (rule 205.4)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Supertype {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
}

/// Subtypes, organized by the card types they can appear on (rule 205.3)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Subtype {
    Artifact(ArtifactType),
    Enchantment(EnchantmentType),
    Land(LandType),
    Creature(CreatureType),
    Planeswalker(PlaneswalkerType),
    Spell(SpellType),
    Planar(PlanarType),
    Dungeon(DungeonType),
    Battle(BattleType),
}

impl Subtype {
    /// The subtype as it is printed — "Zombie", "Power-Plant", "Time
    /// Lord". Read by CR 111.4's default token name, which is built from
    /// the subtypes the creating effect set.
    ///
    /// The variant name, with the three multi-word subtypes in reach spelled
    /// out; a fourth gets a line here rather than a table, since the
    /// variant *is* the word for every other one.
    pub fn word(&self) -> String {
        match self {
            Subtype::Land(LandType::PowerPlant) => "Power-Plant".to_string(),
            Subtype::Land(LandType::Urzas) => "Urza's".to_string(),
            Subtype::Creature(CreatureType::TimeLord) => "Time Lord".to_string(),
            Subtype::Artifact(t) => format!("{t:?}"),
            Subtype::Enchantment(t) => format!("{t:?}"),
            Subtype::Land(t) => format!("{t:?}"),
            Subtype::Creature(t) => format!("{t:?}"),
            Subtype::Planeswalker(t) => format!("{t:?}"),
            Subtype::Spell(t) => format!("{t:?}"),
            Subtype::Planar(t) => format!("{t:?}"),
            Subtype::Dungeon(t) => format!("{t:?}"),
            Subtype::Battle(t) => format!("{t:?}"),
        }
    }
}

/// A set of subtypes (CR 205.3) in type-line order: the printed ones as
/// printed, then each one an effect added, in the order it was added. Only a
/// type line reads the order, so equality ignores it.
///
/// `HashSet`'s method signatures, as [`CardTypes`] keeps them. "Every creature
/// type" (CR 702.73a) is a mark rather than a list, since CR 205.3m's list runs
/// to hundreds of words.
#[derive(Clone, Default)]
pub struct Subtypes {
    listed: Vec<Subtype>,
    every_creature_type: bool,
}

impl Subtypes {
    pub const fn new() -> Subtypes {
        Subtypes { listed: Vec::new(), every_creature_type: false }
    }

    pub fn contains(&self, subtype: &Subtype) -> bool {
        (self.every_creature_type && matches!(subtype, Subtype::Creature(_))) || self.listed.contains(subtype)
    }

    /// Adds `subtype` after those listed, answering whether it was absent.
    pub fn insert(&mut self, subtype: Subtype) -> bool {
        let absent = !self.contains(&subtype);
        if absent {
            self.listed.push(subtype);
        }
        absent
    }

    /// Removes `subtype` from those listed, answering whether it was listed.
    pub fn remove(&mut self, subtype: &Subtype) -> bool {
        let at = self.listed.iter().position(|listed| listed == subtype);
        if let Some(at) = at {
            self.listed.remove(at);
        }
        at.is_some()
    }

    /// CR 702.73a's "is every creature type", answering whether it was not
    /// already. The listed creature types stay listed, for the type line.
    pub fn insert_every_creature_type(&mut self) -> bool {
        let absent = !self.every_creature_type;
        self.every_creature_type = true;
        absent
    }

    pub fn has_every_creature_type(&self) -> bool {
        self.every_creature_type
    }

    /// CR 205.1a: setting subtypes replaces those from each set the new ones
    /// come from (creature types, land types, ...) and keeps the rest, the new
    /// ones listed last in their own order. Setting none clears them all.
    pub fn set(&mut self, new: &Subtypes) {
        if new.is_empty() {
            self.listed.clear();
            self.every_creature_type = false;
            return;
        }
        let same_set = |old: &Subtype| {
            new.listed.iter().any(|n| std::mem::discriminant(n) == std::mem::discriminant(old))
                || (new.every_creature_type && matches!(old, Subtype::Creature(_)))
        };
        self.listed.retain(|old| !same_set(old));
        for subtype in &new.listed {
            if !self.listed.contains(subtype) {
                self.listed.push(subtype.clone());
            }
        }
        if new.every_creature_type || new.listed.iter().any(|n| matches!(n, Subtype::Creature(_))) {
            self.every_creature_type = new.every_creature_type;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.listed.is_empty() && !self.every_creature_type
    }

    /// Those listed, in type-line order; under the every-creature-type mark,
    /// the creature types no effect or card named are not among them.
    pub fn iter(&self) -> std::slice::Iter<'_, Subtype> {
        self.listed.iter()
    }
}

impl PartialEq for Subtypes {
    /// As sets: the mark contains every creature type, listed or not.
    fn eq(&self, other: &Subtypes) -> bool {
        self.every_creature_type == other.every_creature_type
            && self.listed.iter().all(|subtype| other.contains(subtype))
            && other.listed.iter().all(|subtype| self.contains(subtype))
    }
}

impl std::fmt::Debug for Subtypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut set = f.debug_set();
        set.entries(self.listed.iter());
        if self.every_creature_type {
            set.entry(&format_args!("every creature type"));
        }
        set.finish()
    }
}

impl FromIterator<Subtype> for Subtypes {
    fn from_iter<I: IntoIterator<Item = Subtype>>(subtypes: I) -> Subtypes {
        let mut set = Subtypes::new();
        for subtype in subtypes {
            set.insert(subtype);
        }
        set
    }
}

impl<const N: usize> From<[Subtype; N]> for Subtypes {
    fn from(subtypes: [Subtype; N]) -> Subtypes {
        subtypes.into_iter().collect()
    }
}

impl IntoIterator for Subtypes {
    type Item = Subtype;
    type IntoIter = std::vec::IntoIter<Subtype>;
    fn into_iter(self) -> Self::IntoIter {
        self.listed.into_iter()
    }
}

// --- Artifact subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ArtifactType {
    Attraction,
    Blood,
    Bobblehead,
    Clue,
    Contraption,
    Equipment,
    Food,
    Fortification,
    Gold,
    Incubator,
    Infinity,
    Junk,
    Lander,
    Map,
    Mutagen,
    Powerstone,
    Spacecraft,
    Stone,
    Treasure,
    Vehicle,
}

// --- Enchantment subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EnchantmentType {
    Aura,
    Background,
    Cartouche,
    Case,
    Class,
    Curse,
    Role,
    Room,
    Rune,
    Saga,
    Shard,
    Shrine,
}

// --- Land subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LandType {
    // The five basic land types (rule 305.6)
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
    // Non-basic land subtypes
    Cave,
    Desert,
    Gate,
    Lair,
    Locus,
    Mine,
    Planet,
    PowerPlant,
    Sphere,
    Tower,
    Town,
    Urzas,
}

impl LandType {
    /// Whether this is one of the five basic land types (rule 305.6)
    pub fn is_basic_land_type(&self) -> bool {
        matches!(self, LandType::Plains | LandType::Island | LandType::Swamp | LandType::Mountain | LandType::Forest)
    }
}

// --- Creature subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CreatureType {
    Advisor,
    Aetherborn,
    Alien,
    Ally,
    Angel,
    Antelope,
    Ape,
    Archer,
    Archon,
    Armadillo,
    Army,
    Artificer,
    Assassin,
    AssemblyWorker,
    Astartes,
    Atog,
    Aurochs,
    Avatar,
    Azra,
    Badger,
    Balloon,
    Barbarian,
    Bard,
    Basilisk,
    Bat,
    Bear,
    Beast,
    Beaver,
    Beeble,
    Beholder,
    Berserker,
    Bird,
    Blinkmoth,
    Boar,
    Bringer,
    Brushwagg,
    Camarid,
    Camel,
    Capybara,
    Caribou,
    Carrier,
    Cat,
    Centaur,
    Child,
    Chimera,
    Citizen,
    Cleric,
    Clown,
    Cockatrice,
    Construct,
    Coward,
    Coyote,
    Crab,
    Crocodile,
    Ctan,
    Custodes,
    Cyberman,
    Cyclops,
    Dalek,
    Dauthi,
    Demigod,
    Demon,
    Deserter,
    Detective,
    Devil,
    Dinosaur,
    Djinn,
    Doctor,
    Dog,
    Dragon,
    Drake,
    Dreadnought,
    Drone,
    Druid,
    Dryad,
    Dwarf,
    Efreet,
    Egg,
    Elder,
    Eldrazi,
    Elemental,
    Elephant,
    Elf,
    Elk,
    Employee,
    Eye,
    Faerie,
    Ferret,
    Fish,
    Flagbearer,
    Fox,
    Fractal,
    Frog,
    Fungus,
    Gamer,
    Gargoyle,
    Germ,
    Giant,
    Gith,
    Glimmer,
    Gnoll,
    Gnome,
    Goat,
    Goblin,
    God,
    Golem,
    Gorgon,
    Graveborn,
    Gremlin,
    Griffin,
    Guest,
    Hag,
    Halfling,
    Hamster,
    Harpy,
    Hellion,
    Hippo,
    Hippogriff,
    Homarid,
    Homunculus,
    Horror,
    Horse,
    Human,
    Hydra,
    Hyena,
    Illusion,
    Imp,
    Incarnation,
    Inkling,
    Inquisitor,
    Insect,
    Jackal,
    Jellyfish,
    Juggernaut,
    Kavu,
    Kirin,
    Kithkin,
    Knight,
    Kobold,
    Kor,
    Kraken,
    Llama,
    Lamia,
    Lammasu,
    Leech,
    Leviathan,
    Lhurgoyf,
    Licid,
    Lizard,
    Manticore,
    Masticore,
    Mercenary,
    Merfolk,
    Metathran,
    Minion,
    Minotaur,
    Mite,
    Mole,
    Monger,
    Mongoose,
    Monk,
    Monkey,
    Moonfolk,
    Mount,
    Mouse,
    Mutant,
    Myr,
    Mystic,
    Nautilus,
    Necron,
    Nephilim,
    Nightmare,
    Nightstalker,
    Ninja,
    Noble,
    Noggle,
    Nomad,
    Nymph,
    Octopus,
    Ogre,
    Ooze,
    Orb,
    Orc,
    Orgg,
    Otter,
    Ouphe,
    Ox,
    Oyster,
    Pangolin,
    Peasant,
    Pegasus,
    Pentavite,
    Performer,
    Pest,
    Phelddagrif,
    Phoenix,
    Phyrexian,
    Pilot,
    Pincher,
    Pirate,
    Plant,
    Porcupine,
    Possum,
    Praetor,
    Primarch,
    Prism,
    Processor,
    Rabbit,
    Raccoon,
    Ranger,
    Rat,
    Rebel,
    Reflection,
    Rhino,
    Rigger,
    Robot,
    Rogue,
    Sable,
    Salamander,
    Samurai,
    Sand,
    Saproling,
    Satyr,
    Scarecrow,
    Scientist,
    Scion,
    Scorpion,
    Scout,
    Sculpture,
    Seal,
    Serf,
    Serpent,
    Servo,
    Shade,
    Shaman,
    Shapeshifter,
    Shark,
    Sheep,
    Siren,
    Skeleton,
    Skunk,
    Slith,
    Sliver,
    Sloth,
    Slug,
    Snail,
    Snake,
    Soldier,
    Soltari,
    Spawn,
    Specter,
    Spellshaper,
    Sphinx,
    Spider,
    Spike,
    Spirit,
    Splinter,
    Sponge,
    Squid,
    Squirrel,
    Starfish,
    Surrakar,
    Survivor,
    Synth,
    Tentacle,
    Tetravite,
    Thalakos,
    Thopter,
    Thrull,
    Tiefling,
    TimeLord,
    Toy,
    Treefolk,
    Trilobite,
    Triskelavite,
    Troll,
    Turtle,
    Tyranid,
    Unicorn,
    Vampire,
    Varmint,
    Vedalken,
    Volver,
    Wall,
    Walrus,
    Warlock,
    Warrior,
    Weasel,
    Weird,
    Werewolf,
    Whale,
    Wizard,
    Wolf,
    Wolverine,
    Wombat,
    Worm,
    Wraith,
    Wurm,
    Yeti,
    Zombie,
    Zubera,
}

// --- Planeswalker subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlaneswalkerType {
    Ajani,
    Aminatou,
    Angrath,
    Arlinn,
    Ashiok,
    Bahamut,
    Basri,
    Bolas,
    Calix,
    Chandra,
    Comet,
    Dack,
    Dakkon,
    Daretti,
    Davriel,
    Dihada,
    Domri,
    Dovin,
    Ellywick,
    Elminster,
    Elspeth,
    Estrid,
    Freyalise,
    Garruk,
    Gideon,
    Grist,
    Guff,
    Huatli,
    Jace,
    Jared,
    Jaya,
    Jeska,
    Kaito,
    Karn,
    Kasmina,
    Kaya,
    Kiora,
    Koth,
    Liliana,
    Lolth,
    Lukka,
    Minsc,
    Mordenkainen,
    Nahiri,
    Narset,
    Niko,
    Nissa,
    Nixilis,
    Oko,
    Quintorius,
    Ral,
    Rowan,
    Saheeli,
    Samut,
    Sarkhan,
    Serra,
    Sivitri,
    Sorin,
    Szat,
    Tamiyo,
    Tasha,
    Teferi,
    Teyo,
    Tezzeret,
    Tibalt,
    Tyvar,
    Ugin,
    Urza,
    Venser,
    Vivien,
    Vraska,
    Vronos,
    Will,
    Windgrace,
    Wrenn,
    Xenagos,
    Yanggu,
    Yanling,
    Zariel,
}

// --- Spell subtypes (Instant/Sorcery) ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SpellType {
    Adventure,
    Arcane,
    Lesson,
    Omen,
    Trap,
}

// --- Planar subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlanarType {
    TheAbyss,
    Alara,
    AlfavaMetraxis,
    Amonkhet,
    AndrozaniMinor,
    Antausia,
    Apalapucia,
    Arcavios,
    Arkhos,
    Avishkar,
    Azgol,
    Belenon,
    BolassMeditationRealm,
    Capenna,
    Cridhe,
    TheDalekAsylum,
    Darillium,
    Dominaria,
    Earth,
    Echoir,
    Eldraine,
    Equilor,
    Ergamon,
    Fabacin,
    Fiora,
    Gallifrey,
    Gargantikar,
    Gobakhan,
    HorseheadNebula,
    Ikoria,
    Innistrad,
    Iquatana,
    Ir,
    Ixalan,
    Kaldheim,
    Kamigawa,
    Kandoka,
    Karsus,
    Kephalai,
    Kinshala,
    Kolbahan,
    Kylem,
    Kyneth,
    TheLibrary,
    Lorwyn,
    Luvion,
    Mars,
    Mercadia,
    Mirrodin,
    Moag,
    Mongseng,
    Moon,
    Muraganda,
    Necros,
    NewEarth,
    NewPhyrexia,
    OutsideMuttersSpiral,
    Phyrexia,
    Pyrulea,
    Rabiah,
    Rath,
    Ravnica,
    Regatha,
    Segovia,
    SerrasRealm,
    Shadowmoor,
    Shandalar,
    Shenmeng,
    Skaro,
    Spacecraft,
    Tarkir,
    Theros,
    Time,
    Trenzalore,
    Ulgrotha,
    UnknownPlanet,
    Valla,
    Vryn,
    Wildfire,
    Xerex,
    Zendikar,
    Zhalfir,
}

// --- Dungeon subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum DungeonType {
    Undercity,
}

// --- Battle subtypes ---

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BattleType {
    Siege,
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUMAN: Subtype = Subtype::Creature(CreatureType::Human);
    const CLERIC: Subtype = Subtype::Creature(CreatureType::Cleric);
    const DRYAD: Subtype = Subtype::Creature(CreatureType::Dryad);
    const FOREST: Subtype = Subtype::Land(LandType::Forest);
    const MOUNTAIN: Subtype = Subtype::Land(LandType::Mountain);

    fn listed(set: &Subtypes) -> Vec<Subtype> {
        set.iter().cloned().collect()
    }

    #[test]
    fn subtypes_keep_the_order_they_were_added_in_and_compare_as_sets() {
        let mut priest = Subtypes::new();
        assert!(priest.insert(HUMAN));
        assert!(priest.insert(CLERIC));
        assert!(!priest.insert(HUMAN), "already there");
        assert_eq!(listed(&priest), [HUMAN, CLERIC]);
        assert_eq!(priest, Subtypes::from([CLERIC, HUMAN]));
        assert!(priest.remove(&HUMAN));
        assert_eq!(listed(&priest), [CLERIC]);
    }

    #[test]
    fn every_creature_type_is_one_mark_and_no_land_type() {
        let mut changeling = Subtypes::from([Subtype::Creature(CreatureType::Shapeshifter)]);
        assert!(changeling.insert_every_creature_type());
        assert!(changeling.contains(&Subtype::Creature(CreatureType::TimeLord)));
        assert!(!changeling.contains(&FOREST));
        assert!(!changeling.insert(HUMAN), "a creature type it already is");
        assert_eq!(listed(&changeling), [Subtype::Creature(CreatureType::Shapeshifter)]);
        let mut marked = Subtypes::new();
        marked.insert_every_creature_type();
        assert!(!marked.is_empty());
        assert_eq!(changeling, marked, "the listed Shapeshifter is among every creature type");
        assert_eq!(format!("{changeling:?}"), "{Creature(Shapeshifter), every creature type}");
    }

    // COVERS-PARTIAL: ATOM-205.1a-003
    #[test]
    fn setting_subtypes_replaces_only_the_sets_they_come_from() {
        let mut arbor = Subtypes::from([FOREST, DRYAD]);
        arbor.set(&Subtypes::from([MOUNTAIN]));
        assert_eq!(listed(&arbor), [DRYAD, MOUNTAIN], "the land types go, the creature type stays");
        arbor.set(&Subtypes::from([Subtype::Creature(CreatureType::Goblin)]));
        assert_eq!(listed(&arbor), [MOUNTAIN, Subtype::Creature(CreatureType::Goblin)]);

        let mut every = Subtypes::new();
        every.insert_every_creature_type();
        arbor.set(&every);
        assert_eq!(listed(&arbor), [MOUNTAIN]);
        assert!(arbor.has_every_creature_type());
        arbor.set(&Subtypes::from([HUMAN]));
        assert!(!arbor.has_every_creature_type(), "a creature type set replaces every creature type");
        assert_eq!(listed(&arbor), [MOUNTAIN, HUMAN]);
        arbor.set(&Subtypes::new());
        assert!(arbor.is_empty());
    }
}
