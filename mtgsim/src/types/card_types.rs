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
