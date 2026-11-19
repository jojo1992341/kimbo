//! Logique pure (sans interface) : portage exact du calcul de la page HTML.

#[derive(Clone, Copy)]
pub struct State {
    /// Couleur de la case du Kimbo (0 = claire, 1 = foncée)
    pub k: u8,
    /// Couleur de la case du Disciple
    pub d: u8,
    pub idol: bool,
    pub statue: bool,
    /// Ma case (utilisée seulement sans l'idole)
    pub m: u8,
    pub dk: u32,
    pub dd: u32,
}

impl Default for State {
    fn default() -> Self {
        Self { k: 0, d: 0, idol: true, statue: false, m: 0, dk: 3, dd: 3 }
    }
}

pub struct Res {
    pub me: u8,
    pub safe: u8,
    /// true = Air / Feu, false = Terre / Eau
    pub air_feu: bool,
    /// Couleur des cases où le Disciple posera ses glyphes
    pub glyph: u8,
    /// Couleur de la case du Kimbo après son déplacement
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
    // Chronologie d'un tour :
    //  1. je tape le Kimbo (élément choisi) ;
    //  2. le Kimbo bouge d'une case et frappe ;
    //  3. le Disciple pose ses glyphes selon l'élément (Air/Feu : cases de SA couleur,
    //     Terre/Eau : cases de la couleur opposée) PUIS bouge d'une case (+1 PM) ;
    //  4. au début de leur tour, un personnage sur un glyphe meurt, le Kimbo perd ses résistances.
    let k2 = 1 - s.k; // couleur du Kimbo après son déplacement
    let d2 = 1 - s.d; // couleur du Disciple après avoir posé ses glyphes et bougé
    let pull_k = s.idol && s.dk > 8;
    let pull_d = s.idol && s.dd > 8;
    let pulls = pull_k as u8 + pull_d as u8;
    let safe = if s.statue { me } else { s.k ^ (pulls % 2) };
    // Les glyphes sont posés AVANT le déplacement du Disciple : c'est sa couleur actuelle (s.d) qui compte.
    // Statue : je ne bouge pas, les glyphes doivent être sur la couleur opposée à la mienne.
    // Sinon : les glyphes doivent être sur la case où sera le Kimbo (k2).
    let air_feu = if s.statue { s.d == 1 - me } else { k2 == s.d };
    let glyph = if air_feu { s.d } else { 1 - s.d };
    Some(Res {
        me,
        safe,
        air_feu,
        glyph,
        k2,
        d2,
        pull_k,
        pull_d,
        pulls,
        // Statue : si le Kimbo (après son déplacement) n'est pas sur la couleur des glyphes, il faut le pousser
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
        // Kimbo finit sur foncée, Disciple pose depuis claire : il faut des glyphes "impairs" (foncée)
        assert!(!r.air_feu); // Terre / Eau
        assert_eq!(r.glyph, 1);
        assert_eq!(r.glyph, r.k2); // le Kimbo sera sur les glyphes
        assert_ne!(r.glyph, r.safe); // et pas moi
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
        // Idole, Kimbo clair, Disciple clair, 3/3 -> moi = foncée ; glyphes doivent être sur claire
        let s = State { statue: true, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.safe, r.me);
        assert!(r.air_feu); // le Disciple est sur claire : Air/Feu = glyphes sur claire
        assert_eq!(r.glyph, 0);
        assert!(r.statue_push); // le Kimbo finira sur foncée : il faut le pousser
    }

    #[test]
    fn statue_cas_capture() {
        // Sans idole, Kimbo foncé, Disciple clair, moi clair, Succès Statue
        let s = State { k: 1, d: 0, m: 0, idol: false, statue: true, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.safe, 0); // je reste sur claire
        // Air/Feu poserait les glyphes sur claire (= ma case) : je mourrais -> Terre / Eau (foncée)
        assert!(!r.air_feu);
        assert_eq!(r.glyph, 1);
        assert_eq!(r.k2, 0); // le Kimbo finit sur claire (il bouge d'une case)
        assert!(r.statue_push); // il faut le pousser pour qu'il soit sur foncée
        assert_eq!(r.d2, 1);
    }

    #[test]
    fn statue_sans_poussee() {
        // Kimbo clair -> finit sur foncée ; moi clair ; glyphes sur foncée -> aucune poussée
        let s = State { k: 0, d: 0, m: 0, idol: false, statue: true, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.glyph, 1);
        assert!(!r.statue_push);
    }

    #[test]
    fn sans_idole() {
        let s = State { idol: false, m: 1, ..State::default() };
        let r = compute(&s).unwrap();
        assert_eq!(r.me, 1);
        assert_eq!(r.pulls, 0);
    }
}
