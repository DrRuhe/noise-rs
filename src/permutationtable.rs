use core::fmt;
use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_xorshift::XorShiftRng;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};

const TABLE_SIZE: usize = 256;

pub trait NoiseHasher: Send + Sync {
    fn hash(&self, to_hash: &[isize]) -> usize;
}

/// A seed table, required by all noise functions.
///
/// Table creation is expensive, so in most circumstances you'll only want to
/// create one of these per generator.
#[derive(Copy, Clone)]
pub struct PermutationTable {
    values: [u8; TABLE_SIZE],
}

impl serde::Serialize for PermutationTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(TABLE_SIZE))?;
        for value in self.values {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

struct PermutationTableDeserializer;

impl<'de> Visitor<'de> for PermutationTableDeserializer {
    type Value = PermutationTable;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("ArrayKeyedMap key value sequence.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut new_obj = PermutationTable {
            values: [0; TABLE_SIZE],
        };
        for i in 0..TABLE_SIZE {
            if let Some(deserialized_value) = seq.next_element()? {
                new_obj.values[i] = deserialized_value;
            } else {
                return Err(serde::de::Error::custom(format!(
                    "PermutationTable must have exactly {} elements, found {}",
                    TABLE_SIZE, i
                )));
            }
        }

        Ok(new_obj)
    }
}

impl<'de> serde::Deserialize<'de> for PermutationTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(PermutationTableDeserializer)
    }
}

impl Distribution<PermutationTable> for Standard {
    /// Generates a PermutationTable using a random seed.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PermutationTable {
        let mut perm_table = PermutationTable {
            values: [0; TABLE_SIZE],
        };

        perm_table
            .values
            .iter_mut()
            .enumerate()
            .for_each(|(i, b)| *b = i as u8);
        perm_table.values.shuffle(rng);

        perm_table
    }
}

impl PermutationTable {
    /// Deterministically generates a new permutation table based on a `u32` seed value.
    ///
    /// Internally this uses a `XorShiftRng`, but we don't really need to worry
    /// about cryptographic security when working with procedural noise.
    pub fn new(seed: u32) -> Self {
        let mut real = [0; 16];
        real[0] = 1;
        for i in 1..4 {
            real[i * 4] = seed as u8;
            real[(i * 4) + 1] = (seed >> 8) as u8;
            real[(i * 4) + 2] = (seed >> 16) as u8;
            real[(i * 4) + 3] = (seed >> 24) as u8;
        }
        let mut rng: XorShiftRng = SeedableRng::from_seed(real);
        rng.gen()
    }
}

impl NoiseHasher for PermutationTable {
    fn hash(&self, to_hash: &[isize]) -> usize {
        let index = to_hash
            .iter()
            .map(|&a| (a & 0xff) as usize)
            .reduce(|a, b| self.values[a] as usize ^ b)
            .unwrap();
        self.values[index] as usize
    }
}

impl fmt::Debug for PermutationTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PermutationTable {{ .. }}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{NoiseFn, Perlin, Seedable};
    use rand::random;

    #[test]
    fn test_random_seed() {
        let perlin = Perlin::default().set_seed(random());
        let _ = perlin.get([1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_negative_params() {
        let perlin = Perlin::default();
        let _ = perlin.get([-1.0, 2.0, 3.0]);
    }
}
