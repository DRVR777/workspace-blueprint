//! Bone 3c — force-directed layout.
//!
//! Assigns `world_coord: Some([x, y, z])` to every identity file so that
//! the physical trail in `HopRecord.world_coord` stops being `None`.
//!
//! Key insight: we do not need to preserve 384D distances in 3D.
//! We need to preserve 384D nearest-neighbor edges.
//! A layout where every identity's k nearest neighbors in 3D match its
//! k nearest neighbors in 384D is topologically equivalent for routing.
//!
//! Algorithm: force-directed spring layout from HNSW k-NN edges.
//!   - Attractive spring force toward each node's k semantic nearest neighbors.
//!   - Repulsive inverse-square force between all other pairs.
//!   - Converge over N iterations with linear cooling.
//!   - Resulting positions assigned as world_coord.
//!
//! Entry point: `apply_layout(store, k)` — returns a new `IdentityStore`
//! with world_coord set on every file. One HNSW rebuild, not N.

use crate::identity::{IdentityFile, IdentityStore};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Run force-directed layout over `store`, set `world_coord` on every file,
/// and return a new `IdentityStore` built from the updated files.
///
/// `k` — number of attractive nearest-neighbor edges per node (typically 3).
/// One HNSW rebuild at the end. Call once at startup after seeding.
pub fn apply_layout(store: &IdentityStore, k: usize) -> IdentityStore {
    let positions = force_directed_layout(store, k, 150);
    let updated: Vec<IdentityFile> = store.iter().enumerate()
        .map(|(i, f)| IdentityFile {
            address:    f.address.clone(),
            content:    f.content.clone(),
            vector:     f.vector.clone(),
            world_coord: Some(positions[i]),
        })
        .collect();
    IdentityStore::build(updated)
}

// ─── Core algorithm ───────────────────────────────────────────────────────────

/// Compute 3D positions for all identity files via force-directed spring layout.
///
/// Returns one `[f32; 3]` per file, in the same order as `store.iter()`.
pub fn force_directed_layout(
    store:      &IdentityStore,
    k:          usize,
    iterations: u32,
) -> Vec<[f32; 3]> {
    let n = store.len();
    if n == 0 { return vec![]; }
    if n == 1 { return vec![[0.0, 0.0, 0.0]]; }

    // Seed positions on a Fibonacci sphere for uniform initial coverage.
    let mut positions: Vec<[f32; 3]> = (0..n).map(|i| fibonacci_sphere(i, n)).collect();

    // Pre-collect neighbor index lists so we don't re-query HNSW each iteration.
    // Request k+1 to account for the file itself appearing as its own neighbor.
    let neighbor_indices: Vec<Vec<usize>> = store.iter()
        .map(|file| {
            store.nearest_k(&file.vector, k + 1)
                .into_iter()
                .filter_map(|nb| {
                    if nb.address == file.address { None }
                    else { store.index_of(&nb.address) }
                })
                .take(k)
                .collect()
        })
        .collect();

    for iter in 0..iterations {
        // Linear cooling: start at 1.0, finish at 0.1.
        let cool = 1.0 - (iter as f32 / iterations as f32) * 0.9;
        let attract_k = 0.05 * cool;
        let repel_k   = 0.002 * cool;

        let mut forces = vec![[0.0f32; 3]; n];

        for i in 0..n {
            // Attractive: spring toward each k-NN semantic neighbor.
            for &j in &neighbor_indices[i] {
                let delta = sub3(positions[j], positions[i]);
                let dist  = mag3(delta);
                if dist > 1e-8 {
                    // Spring force proportional to distance.
                    let f = scale3(normalize3(delta), dist * attract_k);
                    forces[i] = add3(forces[i], f);
                    forces[j] = sub3(forces[j], f); // Newton's third law
                }
            }

            // Repulsive: inverse-square push away from all other nodes.
            for j in (i + 1)..n {
                let delta = sub3(positions[i], positions[j]);
                let dist  = mag3(delta).max(0.05); // clamp to prevent singularity
                let magnitude = repel_k / (dist * dist);
                let f = scale3(normalize3(delta), magnitude);
                forces[i] = add3(forces[i], f);
                forces[j] = sub3(forces[j], f);
            }
        }

        // Apply forces. Clamp step size to prevent instability.
        for i in 0..n {
            let step = clamp_mag(forces[i], 0.1 * cool);
            positions[i] = add3(positions[i], step);
        }
    }

    positions
}

// ─── Vector math helpers ─────────────────────────────────────────────────────

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3]  { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3]  { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn scale3(a: [f32; 3], s: f32) -> [f32; 3]     { [a[0]*s, a[1]*s, a[2]*s] }
fn mag3(a: [f32; 3]) -> f32                    { (a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt() }

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let m = mag3(a);
    if m > 1e-8 { scale3(a, 1.0 / m) } else { [0.0, 0.0, 0.0] }
}

/// Clamp a force vector to a maximum magnitude.
fn clamp_mag(v: [f32; 3], max_mag: f32) -> [f32; 3] {
    let m = mag3(v);
    if m > max_mag { scale3(normalize3(v), max_mag) } else { v }
}

/// Fibonacci lattice on a unit sphere — uniform distribution of N points.
fn fibonacci_sphere(i: usize, n: usize) -> [f32; 3] {
    if n == 1 { return [0.0, 0.0, 0.0]; }
    let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());
    let y      = 1.0 - (i as f32 / (n as f32 - 1.0)) * 2.0;
    let radius = (1.0 - y * y).max(0.0).sqrt();
    let theta  = golden * i as f32;
    [radius * theta.cos(), y, radius * theta.sin()]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::seed_identities;

    fn euclidean3(a: [f32; 3], b: [f32; 3]) -> f32 {
        mag3(sub3(a, b))
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32   = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let ma: f32    = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mb: f32    = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if ma == 0.0 || mb == 0.0 { 0.0 } else { dot / (ma * mb) }
    }

    #[test]
    fn all_seeds_get_world_coord() {
        let store  = IdentityStore::build(seed_identities());
        let laid   = apply_layout(&store, 3);

        assert_eq!(laid.len(), 5);
        for file in laid.iter() {
            assert!(
                file.world_coord.is_some(),
                "expected world_coord on {}", file.address
            );
        }
    }

    #[test]
    fn positions_are_finite_and_non_degenerate() {
        let store  = IdentityStore::build(seed_identities());
        let laid   = apply_layout(&store, 3);

        let positions: Vec<[f32; 3]> = laid.iter()
            .map(|f| f.world_coord.unwrap())
            .collect();

        // All finite
        for (i, &p) in positions.iter().enumerate() {
            for (axis, v) in p.iter().enumerate() {
                assert!(v.is_finite(), "position[{i}][{axis}] = {v} is not finite");
            }
        }

        // Not all at the same point — layout actually spread them
        let first = positions[0];
        let any_differs = positions.iter().skip(1).any(|&p| euclidean3(p, first) > 1e-4);
        assert!(any_differs, "all positions collapsed to the same point");
    }

    #[test]
    fn semantic_neighbors_are_3d_neighbors() {
        // PHILOSOPHER [0.2, 0.1, 0.8, 0.9, 0.7] and ENGINEER [0.9, 0.9, 0.2, 0.5, 0.9]
        // are the most dissimilar pair in the seed set.
        // SYNTHESIZER [0.5, 0.3, 0.5, 1.0, 0.6] and OBSERVER [0.4, 0.2, 0.3, 0.7, 0.95]
        // are more similar.
        // After layout, SYNTH–OBSERVER 3D distance should be < PHIL–ENG 3D distance.

        let seeds = seed_identities();
        let philosopher_v = seeds.iter().find(|f| f.address.contains("PHILOSOPHER")).unwrap().vector.clone();
        let engineer_v    = seeds.iter().find(|f| f.address.contains("ENGINEER")).unwrap().vector.clone();
        let synthesizer_v = seeds.iter().find(|f| f.address.contains("SYNTHESIZER")).unwrap().vector.clone();
        let observer_v    = seeds.iter().find(|f| f.address.contains("OBSERVER")).unwrap().vector.clone();

        let phil_eng_sim   = cosine_sim(&philosopher_v, &engineer_v);
        let synth_obs_sim  = cosine_sim(&synthesizer_v, &observer_v);
        assert!(
            synth_obs_sim > phil_eng_sim,
            "test precondition: SYNTH-OBS ({synth_obs_sim:.3}) should be more similar than PHIL-ENG ({phil_eng_sim:.3})"
        );

        let store = IdentityStore::build(seed_identities());
        let laid  = apply_layout(&store, 3);

        let pos = |name: &str| -> [f32; 3] {
            laid.iter()
                .find(|f| f.address.contains(name))
                .unwrap()
                .world_coord
                .unwrap()
        };

        let d_phil_eng   = euclidean3(pos("PHILOSOPHER"), pos("ENGINEER"));
        let d_synth_obs  = euclidean3(pos("SYNTHESIZER"), pos("OBSERVER"));

        assert!(
            d_synth_obs < d_phil_eng,
            "expected SYNTH-OBS 3D dist ({d_synth_obs:.4}) < PHIL-ENG ({d_phil_eng:.4})"
        );
    }
}
