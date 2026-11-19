//! Logique pure (sans interface) : portage exact du calcul de la page HTML.

#[derive(Clone, Copy)]
pub struct State {
    /// Couleur de la case du Kimbo (0 = claire, 1 = foncée)
    pub k: u8,
    /// Couleur de la case du Disciple
    pub d: u8,
    /// Glyphes déclenchés au tour d'avant (1 = oui)
    pub o: u8,
    pub idol: bool,
    pub statue: bool,
    /// Ma case (utilisée seulement sans l'idole)
    pub m: u8,
    pub dk: u32,
    pub dd: u32,
}

impl Default for State {
    fn default() -> Self {
        Self { k: 0, d: 0, o: 1, idol: true, statue: false, m: 0, dk: 3, dd: 3 }
    }
}

pub struct Res {
    pub me: u8,
    pub safe: u8,
    /// true = Air / Feu, false = Terre / Eau
    pub air_feu: bool,
    pub k2: u8,
    pub d2: u8,
    pub pull_k: bool,
    pub pull_d: bool,
    pub pulls: u8,
    pub statue_push: bool,
}

pub fn compute(s: &State) -> Option<Res> {
    let me = if s.idol {
        let mk = ((s.k as u32 + s.dk) % 2) as u8;
        let md = ((s.d as u32 + s.dd) % 2) as u8;
        if mk != md {
            return None; // infos incohérentes
        }
        mk
    } else {
        s.m
    };
    let k2 = 1 - s.k;
    let d2 = s.d ^ s.o;
    let pull_k = s.idol && s.dk > 8;
    let pull_d = s.idol && s.dd > 8;
    let pulls = pull_k as u8 + pull_d as u8;
    let safe = if s.statue { me } else { s.k ^ (pulls % 2) };
    let same = if s.statue { d2 == 1 - me } else { k2 == d2 };
    Some(Res {
        me,
        safe,
        air_feu: same,
        k2,
        d2,
        pull_k,
        pull_d,
        pulls,
        statue_push: s.statue && k2 != 1 - me,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaut() {
        // idole, Kimbo clair, Disciple clair, distances 3/3 -> moi = foncée
        let r = compute(&State::default()).unwrap();
        assert_eq!(r.me, 1);
        assert_eq!(r.safe, 0); // se placer sur Claire
        assert_eq!((r.k2, r.d2), (1, 1));
        assert!(r.air_feu); // Air / Feu
    }

    #[test]
    fn incoherent() {
        let s = State { dk: 3, dd: 4, ..State::default() };
        assert!(compute(&s).is_none());
    }

    #[test]
    fn attirance_unique() {
        let s = State { dk: 9, dd: 3, ..State::default() }; // 0+9=1, 0+3=1 -> cohérent
        let r = compute(&s).unwrap();
        assert_eq!(r.pulls, 1);
        assert_eq!(r.safe, 1);
    }

    #[test]
    fn statue() {
        let s = State { statue: true, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.safe, r.me);
        assert!(r.statue_push); // k2=1 != 1-me=0, comme dans la version HTML
    }

    #[test]
    fn sans_idole() {
        let s = State { idol: false, m: 1, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.me, 1);
        assert_eq!(r.pulls, 0);
    }
}
