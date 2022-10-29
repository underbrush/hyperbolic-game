use nalgebra::Matrix4;
use super::SQ3;

#[derive(Clone, Copy)]
pub struct Permutation {
    func: [usize; 9]
}

#[derive(Clone, Copy)]
pub struct GroupElt {
    perm: Permutation,
    matrix: Matrix4<f32>,
    pub id: Option<u32>,
}

// R: (1 2)(3 4 8 7)(5 6)
pub const ROTATION: GroupElt =
    GroupElt {
        perm: Permutation { func: [0, 2, 1, 4, 8, 6, 5, 3, 7] },
        matrix: Matrix4::new(
            0.0, 1.0, 0.0, 0.0,
            -1., 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ),
        id: Some(0),
    };

// T: (0 1)(2 8 3)(4 5 7 6)
pub const TRANSLATION: GroupElt =
    GroupElt {
        perm: Permutation { func: [1, 0, 8, 2, 5, 7, 4, 6, 3] },
        matrix: Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, SQ3,
            0.0, 0.0, 1.0, 0.0,
            0.0, SQ3, 0.0, 2.0
        ),
        id: Some(5167),
    };

// e: ()
pub const IDENTITY: GroupElt =
    GroupElt {
        perm: Permutation::identity(),
        matrix: Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ),
        id: Some(0),
    };

impl Permutation {
    const fn identity() -> Permutation {
        Self { func: [0, 1, 2, 3, 4, 5, 6, 7, 8] }
    }

    const fn rotation() -> Permutation {
        Self { func: [0, 2, 1, 4, 8, 6, 5, 3, 7] }
    }

    fn multiply(&self, other: &Permutation) -> Permutation {
        let mut out = [0usize; 9];
        for i in 0..9 {
            out[i] = self.func[other.func[i]];
        }
        return Permutation {func: out}
    }

    // fn left_multiply_in_place(&mut self, other: &Permutation) {
    //     let mut out = [0usize; 9];
    //     for i in 0..9 {
    //         out[i] = other.func[self.func[i]];
    //     }
    //     self.func = out;
    // }

    fn right_multiply_in_place(&mut self, other: &Permutation) {
        let mut out = [0usize; 9];
        for i in 0..9 {
            out[i] = self.func[other.func[i]];
        }
        self.func = out;
    }

    fn to_int(&self) -> u32 {
        let mut out = 0u32;
        let mut factor = 1u32;
        for i in 0..9 {
            let mut more_than = 0u32;
            for j in 0..i {
                if self.func[j] > self.func[i] {
                    more_than += 1;
                }
            }
            out += more_than * factor;
            factor *= (i + 1) as u32;
        }
        out
    }

    fn repr(&self) -> u32 {
        let mut out = self.to_int();
        let mut current = self.multiply(&Permutation::identity());
        for _ in 0..3 {
            current.right_multiply_in_place(&Self::rotation());
            let x = current.to_int();
            if x < out {
                out = x;
            }
        }
        out

    }
}

impl GroupElt {
    pub fn multiply(&self, other: &GroupElt) -> GroupElt {
        GroupElt {
            perm: self.perm.multiply(&other.perm),
            matrix: self.matrix * other.matrix,
            id: None
        }
    }

    pub fn right_multiply_in_place(&mut self, other: &GroupElt) {
        self.perm.right_multiply_in_place(&other.perm);
        self.matrix = self.matrix * other.matrix;

        if other.id != Some(0) {
            self.id = None;
        }
    }

    // pub fn left_multiply_in_place(&mut self, other: &GroupElt) {
    //     self.perm.left_multiply_in_place(&other.perm);
    //     self.matrix = other.matrix * self.matrix;

    //     if other.id != Some(0) {
    //         self.id = None;
    //     }
    // }

    pub fn make_repr(&mut self) {
        //println!("{:?}", self.perm.func);
        let id = self.get_id();
        while self.perm.to_int() != id {
            self.right_multiply_in_place(&ROTATION);
        }
    }

    pub fn get_matrix(&self) -> Matrix4<f32> { self.matrix }
    pub fn get_id(&mut self) -> u32 {
        match self.id {
            None => {self.id = Some(self.perm.repr()); self.id.unwrap()},
            Some(n) => n,
        }
    }

    pub fn permute_only(&self) -> GroupElt {
        GroupElt { perm: self.perm, matrix: Matrix4::identity(), id: self.id }
    }
}