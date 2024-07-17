use cubecl_core as cubecl;
use cubecl_core::prelude::*;

use super::{
    base::{Accumulators, Dimensions, Offsets},
    config::CmmaConfig,
};

#[cube]
pub(crate) fn write_to_output<F: Float>(
    out: &mut Tensor<F>,
    accumulators: Accumulators<F>,
    offsets: Offsets,
    dims: Dimensions,
    config: Comptime<CmmaConfig>,
) {
    let acc_sm = fragment_to_shared_memory(accumulators);
    shared_memory_to_output(out, offsets, acc_sm, dims, config);
}

// #[cube]
// pub(crate) fn write_to_output<F: Float>(
//     out: &mut Tensor<F>,
//     accumulators: Accumulators<F>,
//     offsets: Offsets,
//     dims: Dimensions,
//     config: Comptime<CmmaConfig>,
// ) {
//     let block_size_k = Comptime::map(config, |c| c.block_size_k);
//     let block_size_n = Comptime::map(config, |c| c.block_size_k);

//     let tile_size = Comptime::map(config, |c| c.tile_size);
//     let tile_size_r = Comptime::runtime(tile_size);
//     let out_vec = Comptime::vectorization(out);
//     let out_vec_r = Comptime::runtime(out_vec);

//     let num_tiles_per_subcube = Comptime::runtime(block_size_k / tile_size); // 2
//     let acc_sm_stride = Comptime::runtime(block_size_n); // 64
//     let acc_sm_stride_vec = Comptime::runtime(block_size_n / out_vec); // Even if not really vectorized, because write out_vec_r values

//     let out_stride = dims.n;

//     let subcube_dim = UInt::new(32);
//     let within_tile_row_offset = subcube_dim / out_vec_r; // assuming subcube_dim is 32 -> 8
//     let within_sm_row_offset = subcube_dim * out_vec_r / acc_sm_stride; // assuming subcube_dim is 32 -> 2
//     let subcube_id = UNIT_POS_Y;
//     let id_within_subcube = UNIT_POS_X; // lane_id

//     // There are two because 32 / 16. TODO generalize
//     let unit_read_row_0 = id_within_subcube / acc_sm_stride_vec;
//     let unit_read_row_1 = unit_read_row_0 + within_sm_row_offset;
//     let unit_read_col = id_within_subcube % acc_sm_stride_vec;

//     let n_units_per_tile_row = Comptime::runtime(tile_size / out_vec); // 4
//     let unit_write_row_0 = id_within_subcube / n_units_per_tile_row;
//     let unit_write_row_1 = unit_write_row_0 + within_tile_row_offset;
//     let unit_write_col = id_within_subcube % n_units_per_tile_row;

//     // TODO: the need for this shared memory should be replaced by using __shfl_sync
//     // 4096 = 256 * 2 * 8 = content of accumulator * 2 accumulators * 8 warps in parallel = 64 * 64 as well
//     let mut acc_sm = SharedMemory::<F>::new(4096);

//     // for n_iter in range(0u32, num_tiles_per_subcube, Comptime::new(true)) { // MANUAL UNROLL
//     let n_iter = UInt::new(0);
//     let num_slice = UInt::new(2) * subcube_id + n_iter;
//     let slice = acc_sm.slice_mut(
//         num_slice * UInt::new(256),
//         (num_slice + UInt::new(1)) * UInt::new(256),
//     );
//     cmma::store::<F>(
//         slice,
//         &accumulators.first,
//         UInt::new(16),
//         cmma::MatrixLayout::RowMajor,
//     );

//     let single_row_offset = Comptime::runtime(tile_size * tile_size / block_size_n); // 4
//     let row_offset = (num_tiles_per_subcube * subcube_id + n_iter) * single_row_offset;

//     let read_pos_0 = (row_offset + unit_read_row_0) * acc_sm_stride + unit_read_col * out_vec_r;
//     let read_pos_1 = (row_offset + unit_read_row_1) * acc_sm_stride + unit_read_col * out_vec_r;

//     let tile_row = subcube_id / num_tiles_per_subcube;
//     let tile_col = (subcube_id % num_tiles_per_subcube) * num_tiles_per_subcube + n_iter;

//     let total_col = tile_col * n_units_per_tile_row + unit_write_col;

//     let out_offset = offsets.batch_out + offsets.cube_row * out_stride + offsets.cube_col;

//     let out_write_pos_0 = out_offset
//         + (tile_row * tile_size_r + unit_write_row_0) * out_stride
//         + total_col * out_vec_r;
//     let out_write_pos_1 = out_offset
//         + (tile_row * tile_size_r + unit_write_row_1) * out_stride
//         + total_col * out_vec_r;

//     // TODO use store instruction directly
//     let mut a = F::vectorized_empty(Comptime::get(out_vec));
//     for i in range(0u32, 4u32, Comptime::new(true)) {
//         a[i] = acc_sm[read_pos_0 + i];
//     }
//     out[out_write_pos_0 / out_vec_r] = a;

//     let mut b = F::vectorized_empty(Comptime::get(out_vec));
//     for i in range(0u32, 4u32, Comptime::new(true)) {
//         b[i] = acc_sm[read_pos_1 + i];
//     }
//     out[out_write_pos_1 / out_vec_r] = b;

//     /////

//     let n_iter = UInt::new(1);
//     let num_slice = UInt::new(2) * subcube_id + n_iter;
//     let slice = acc_sm.slice_mut(
//         num_slice * UInt::new(256),
//         (num_slice + UInt::new(1)) * UInt::new(256),
//     );
//     cmma::store::<F>(
//         slice,
//         &accumulators.second,
//         UInt::new(16),
//         cmma::MatrixLayout::RowMajor,
//     );

//     let single_row_offset = Comptime::runtime(tile_size * tile_size / block_size_n); // 4
//     let row_offset = (num_tiles_per_subcube * subcube_id + n_iter) * single_row_offset;

//     let read_pos_0 = (row_offset + unit_read_row_0) * acc_sm_stride + unit_read_col * out_vec_r;
//     let read_pos_1 = (row_offset + unit_read_row_1) * acc_sm_stride + unit_read_col * out_vec_r;

//     let tile_row = subcube_id / num_tiles_per_subcube;
//     let tile_col = (subcube_id % num_tiles_per_subcube) * num_tiles_per_subcube + n_iter;

//     let total_col = tile_col * n_units_per_tile_row + unit_write_col;

//     let out_offset = offsets.batch_out + offsets.cube_row * out_stride + offsets.cube_col;

//     let out_write_pos_0 = out_offset
//         + (tile_row * tile_size_r + unit_write_row_0) * out_stride
//         + total_col * out_vec_r;
//     let out_write_pos_1 = out_offset
//         + (tile_row * tile_size_r + unit_write_row_1) * out_stride
//         + total_col * out_vec_r;

//     // TODO use store instruction directly
//     let mut aa = F::vectorized_empty(Comptime::get(out_vec));
//     for i in range(0u32, 4u32, Comptime::new(true)) {
//         aa[i] = acc_sm[read_pos_0 + i];
//     }
//     out[out_write_pos_0 / out_vec_r] = aa;

//     let mut bb = F::vectorized_empty(Comptime::get(out_vec));
//     for i in range(0u32, 4u32, Comptime::new(true)) {
//         bb[i] = acc_sm[read_pos_1 + i];
//     }
//     out[out_write_pos_1 / out_vec_r] = bb;
// }

#[cube]
fn fragment_to_shared_memory<F: Float>(accumulators: Accumulators<F>) -> SharedMemory<F> {
    // TODO: the need for this shared memory should be replaced by using __shfl_sync

    // 4096 = 256 * 2 * 8 = content of accumulator * 2 accumulators * 8 warps in parallel = 64 * 64 as well
    let mut acc_sm = SharedMemory::<F>::new(4096);
    let subcube_id = UNIT_POS_Y;

    let slice_offset_0 = UInt::new(2) * subcube_id * UInt::new(256);
    let slice_offset_1 = slice_offset_0 + UInt::new(256);
    let slice_offset_2 = slice_offset_1 + UInt::new(256);

    let slice = acc_sm.slice_mut(slice_offset_0, slice_offset_1);
    cmma::store::<F>(
        slice,
        &accumulators.first,
        UInt::new(16),
        cmma::MatrixLayout::RowMajor,
    );

    let slice = acc_sm.slice_mut(slice_offset_1, slice_offset_2);
    cmma::store::<F>(
        slice,
        &accumulators.second,
        UInt::new(16),
        cmma::MatrixLayout::RowMajor,
    );

    acc_sm
}

#[cube]
fn shared_memory_to_output<F: Float>(
    out: &mut Tensor<F>,
    offsets: Offsets,
    accumulator_sm: SharedMemory<F>,
    dims: Dimensions,
    config: Comptime<CmmaConfig>,
) {
    let block_size_k = Comptime::map(config, |c| c.block_size_k);
    let block_size_n = Comptime::map(config, |c| c.block_size_k);

    let tile_size = Comptime::map(config, |c| c.tile_size);
    let tile_size_r = Comptime::runtime(tile_size);
    let out_vec = Comptime::vectorization(out);
    let out_vec_r = Comptime::runtime(out_vec);

    let num_tiles_per_subcube = Comptime::runtime(block_size_k / tile_size); // 2
    let acc_sm_stride = Comptime::runtime(block_size_n); // 64
    let acc_sm_stride_vec = Comptime::runtime(block_size_n / out_vec); // Even if not really vectorized, because write out_vec_r values

    let out_stride = dims.n;

    let subcube_dim = UInt::new(32);
    let within_tile_row_offset = subcube_dim / out_vec_r; // assuming subcube_dim is 32 -> 8
    let within_sm_row_offset = subcube_dim * out_vec_r / acc_sm_stride; // assuming subcube_dim is 32 -> 2
    let subcube_id = UNIT_POS_Y;
    let id_within_subcube = UNIT_POS_X; // lane_id

    // There are two because 32 / 16. TODO generalize
    let unit_read_row_0 = id_within_subcube / acc_sm_stride_vec;
    let unit_read_row_1 = unit_read_row_0 + within_sm_row_offset;
    let unit_read_col = id_within_subcube % acc_sm_stride_vec;

    let n_units_per_tile_row = Comptime::runtime(tile_size / out_vec); // 4
    let unit_write_row_0 = id_within_subcube / n_units_per_tile_row;
    let unit_write_row_1 = unit_write_row_0 + within_tile_row_offset;
    let unit_write_col = id_within_subcube % n_units_per_tile_row;

    let n_iter = UInt::new(0);

    let single_row_offset = Comptime::runtime(tile_size * tile_size / block_size_n); // 4
    let row_offset = (num_tiles_per_subcube * subcube_id + n_iter) * single_row_offset;

    let read_pos_0 = (row_offset + unit_read_row_0) * acc_sm_stride + unit_read_col * out_vec_r;
    let read_pos_1 = (row_offset + unit_read_row_1) * acc_sm_stride + unit_read_col * out_vec_r;

    let tile_row = subcube_id / num_tiles_per_subcube;
    let tile_col = (subcube_id % num_tiles_per_subcube) * num_tiles_per_subcube + n_iter;

    let total_col = tile_col * n_units_per_tile_row + unit_write_col;

    let out_offset = offsets.batch_out + offsets.cube_row * out_stride + offsets.cube_col;

    let out_write_pos_0 = out_offset
        + (tile_row * tile_size_r + unit_write_row_0) * out_stride
        + total_col * out_vec_r;
    let out_write_pos_1 = out_offset
        + (tile_row * tile_size_r + unit_write_row_1) * out_stride
        + total_col * out_vec_r;

    // TODO use store instruction directly
    let mut a = F::vectorized_empty(Comptime::get(out_vec));
    for i in range(0u32, 4u32, Comptime::new(true)) {
        a[i] = accumulator_sm[read_pos_0 + i];
    }
    out[out_write_pos_0 / out_vec_r] = a;

    let mut b = F::vectorized_empty(Comptime::get(out_vec));
    for i in range(0u32, 4u32, Comptime::new(true)) {
        b[i] = accumulator_sm[read_pos_1 + i];
    }
    out[out_write_pos_1 / out_vec_r] = b;

    ////

    let n_iter = UInt::new(1);

    let single_row_offset = Comptime::runtime(tile_size * tile_size / block_size_n); // 4
    let row_offset = (num_tiles_per_subcube * subcube_id + n_iter) * single_row_offset;

    let read_pos_0 = (row_offset + unit_read_row_0) * acc_sm_stride + unit_read_col * out_vec_r;
    let read_pos_1 = (row_offset + unit_read_row_1) * acc_sm_stride + unit_read_col * out_vec_r;

    let tile_row = subcube_id / num_tiles_per_subcube;
    let tile_col = (subcube_id % num_tiles_per_subcube) * num_tiles_per_subcube + n_iter;

    let total_col = tile_col * n_units_per_tile_row + unit_write_col;

    let out_offset = offsets.batch_out + offsets.cube_row * out_stride + offsets.cube_col;

    let out_write_pos_0 = out_offset
        + (tile_row * tile_size_r + unit_write_row_0) * out_stride
        + total_col * out_vec_r;
    let out_write_pos_1 = out_offset
        + (tile_row * tile_size_r + unit_write_row_1) * out_stride
        + total_col * out_vec_r;

    // TODO use store instruction directly
    let mut aa = F::vectorized_empty(Comptime::get(out_vec));
    for i in range(0u32, 4u32, Comptime::new(true)) {
        aa[i] = accumulator_sm[read_pos_0 + i];
    }
    out[out_write_pos_0 / out_vec_r] = aa;

    let mut bb = F::vectorized_empty(Comptime::get(out_vec));
    for i in range(0u32, 4u32, Comptime::new(true)) {
        bb[i] = accumulator_sm[read_pos_1 + i];
    }
    out[out_write_pos_1 / out_vec_r] = bb;
}

#[cfg(feature = "export_tests")]
/// Compute loop exported tests
pub mod tests {

    use crate::matmul::{
        cmma::base::{DimensionsExpand, OffsetsExpand},
        test_utils::{assert_equals, assert_equals_range, range_tensor, zeros_tensor},
    };

    use super::*;

    #[cube(launch)]
    fn write_output_test<F: Float>(
        out: &mut Tensor<F>,
        acc_sm_arr: &mut Array<F>, // TODO can't have a non-mut array?
        k: UInt,
        n: UInt,
        config: Comptime<CmmaConfig>,
    ) {
        let offsets = Offsets {
            batch_lhs: UInt::new(0),
            batch_rhs: UInt::new(0),
            batch_out: UInt::new(0),
            cube_row: UInt::new(0),
            cube_col: UInt::new(0),
            k: UInt::new(0),
        };

        let mut accumulate = SharedMemory::<F>::new(4096);
        for i in range(0u32, 4096u32, Comptime::new(false)) {
            accumulate[i] = acc_sm_arr[i];
        }

        let dims = Dimensions {
            m: UInt::new(16),
            k,
            n,
        };

        shared_memory_to_output(out, offsets, accumulate, dims, config);
    }

    /// Exported test
    pub fn cmma_write_output_unit_test<R: Runtime>(device: &R::Device) {
        let k = 16;
        let n = 32;
        let out = zeros_tensor::<R>(k, n, device);
        let acc_sm = range_tensor::<R>(64, 64, device);
        let cube_dim = CubeDim::new(1, 1, 1);
        let cube_count: CubeCount<R::Server> = CubeCount::Static(1, 1, 1);

        let config = CmmaConfig {
            block_size_m: UInt::new(64),
            block_size_k: UInt::new(32),
            block_size_n: UInt::new(64),
            check_m_bounds: false,
            check_k_bounds: false,
            check_n_bounds: false,
            tile_size: UInt::new(16),
            sm_vec: UInt::new(4),
            lhs_transposed: false,
            rhs_transposed: false,
            unroll: false,
        };

        write_output_test::launch::<F32, R>(
            R::client(device),
            cube_count,
            cube_dim,
            TensorArg::vectorized(4, &out.handle, &out.strides, &out.shape),
            ArrayArg::new(&acc_sm.handle, 64 * 64),
            ScalarArg::new(k as u32),
            ScalarArg::new(n as u32),
            config,
        );

        let expected = &[
            0.0, 1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 256.0,
            257.0, 258.0, 259.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 128.0, 129.0, 130.0, 131.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 384.0, 385.0, 386.0, 387.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        assert_equals::<R>(out.handle, expected, device);
    }

    /// Exported test
    pub fn cmma_write_output_warp_test<R: Runtime>(device: &R::Device) {
        let k = 16;
        let n = 32;
        let out = range_tensor::<R>(k, n, device);
        let acc_sm = range_tensor::<R>(64, 64, device);
        let cube_dim = CubeDim::new(32, 1, 1);
        let cube_count: CubeCount<R::Server> = CubeCount::Static(1, 1, 1);

        let config = CmmaConfig {
            block_size_m: UInt::new(64),
            block_size_k: UInt::new(32),
            block_size_n: UInt::new(64),
            check_m_bounds: false,
            check_k_bounds: false,
            check_n_bounds: false,
            tile_size: UInt::new(16),
            sm_vec: UInt::new(4),
            lhs_transposed: false,
            rhs_transposed: false,
            unroll: false,
        };

        write_output_test::launch::<F32, R>(
            R::client(device),
            cube_count,
            cube_dim,
            TensorArg::vectorized(4, &out.handle, &out.strides, &out.shape),
            ArrayArg::new(&acc_sm.handle, 64 * 64),
            ScalarArg::new(k as u32),
            ScalarArg::new(n as u32),
            config,
        );

        let expected = &[
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            256.0, 257.0, 258.0, 259.0, 260.0, 261.0, 262.0, 263.0, 264.0, 265.0, 266.0, 267.0,
            268.0, 269.0, 270.0, 271.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
            26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 272.0, 273.0, 274.0, 275.0, 276.0, 277.0, 278.0,
            279.0, 280.0, 281.0, 282.0, 283.0, 284.0, 285.0, 286.0, 287.0, 32.0, 33.0, 34.0, 35.0,
            36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 288.0, 289.0,
            290.0, 291.0, 292.0, 293.0, 294.0, 295.0, 296.0, 297.0, 298.0, 299.0, 300.0, 301.0,
            302.0, 303.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0,
            60.0, 61.0, 62.0, 63.0, 304.0, 305.0, 306.0, 307.0, 308.0, 309.0, 310.0, 311.0, 312.0,
            313.0, 314.0, 315.0, 316.0, 317.0, 318.0, 319.0, 64.0, 65.0, 66.0, 67.0, 68.0, 69.0,
            70.0, 71.0, 72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0, 320.0, 321.0, 322.0, 323.0,
            324.0, 325.0, 326.0, 327.0, 328.0, 329.0, 330.0, 331.0, 332.0, 333.0, 334.0, 335.0,
            80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0, 93.0,
            94.0, 95.0, 336.0, 337.0, 338.0, 339.0, 340.0, 341.0, 342.0, 343.0, 344.0, 345.0,
            346.0, 347.0, 348.0, 349.0, 350.0, 351.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0,
            103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 352.0, 353.0, 354.0,
            355.0, 356.0, 357.0, 358.0, 359.0, 360.0, 361.0, 362.0, 363.0, 364.0, 365.0, 366.0,
            367.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0,
            123.0, 124.0, 125.0, 126.0, 127.0, 368.0, 369.0, 370.0, 371.0, 372.0, 373.0, 374.0,
            375.0, 376.0, 377.0, 378.0, 379.0, 380.0, 381.0, 382.0, 383.0, 128.0, 129.0, 130.0,
            131.0, 132.0, 133.0, 134.0, 135.0, 136.0, 137.0, 138.0, 139.0, 140.0, 141.0, 142.0,
            143.0, 384.0, 385.0, 386.0, 387.0, 388.0, 389.0, 390.0, 391.0, 392.0, 393.0, 394.0,
            395.0, 396.0, 397.0, 398.0, 399.0, 144.0, 145.0, 146.0, 147.0, 148.0, 149.0, 150.0,
            151.0, 152.0, 153.0, 154.0, 155.0, 156.0, 157.0, 158.0, 159.0, 400.0, 401.0, 402.0,
            403.0, 404.0, 405.0, 406.0, 407.0, 408.0, 409.0, 410.0, 411.0, 412.0, 413.0, 414.0,
            415.0, 160.0, 161.0, 162.0, 163.0, 164.0, 165.0, 166.0, 167.0, 168.0, 169.0, 170.0,
            171.0, 172.0, 173.0, 174.0, 175.0, 416.0, 417.0, 418.0, 419.0, 420.0, 421.0, 422.0,
            423.0, 424.0, 425.0, 426.0, 427.0, 428.0, 429.0, 430.0, 431.0, 176.0, 177.0, 178.0,
            179.0, 180.0, 181.0, 182.0, 183.0, 184.0, 185.0, 186.0, 187.0, 188.0, 189.0, 190.0,
            191.0, 432.0, 433.0, 434.0, 435.0, 436.0, 437.0, 438.0, 439.0, 440.0, 441.0, 442.0,
            443.0, 444.0, 445.0, 446.0, 447.0, 192.0, 193.0, 194.0, 195.0, 196.0, 197.0, 198.0,
            199.0, 200.0, 201.0, 202.0, 203.0, 204.0, 205.0, 206.0, 207.0, 448.0, 449.0, 450.0,
            451.0, 452.0, 453.0, 454.0, 455.0, 456.0, 457.0, 458.0, 459.0, 460.0, 461.0, 462.0,
            463.0, 208.0, 209.0, 210.0, 211.0, 212.0, 213.0, 214.0, 215.0, 216.0, 217.0, 218.0,
            219.0, 220.0, 221.0, 222.0, 223.0, 464.0, 465.0, 466.0, 467.0, 468.0, 469.0, 470.0,
            471.0, 472.0, 473.0, 474.0, 475.0, 476.0, 477.0, 478.0, 479.0, 224.0, 225.0, 226.0,
            227.0, 228.0, 229.0, 230.0, 231.0, 232.0, 233.0, 234.0, 235.0, 236.0, 237.0, 238.0,
            239.0, 480.0, 481.0, 482.0, 483.0, 484.0, 485.0, 486.0, 487.0, 488.0, 489.0, 490.0,
            491.0, 492.0, 493.0, 494.0, 495.0, 240.0, 241.0, 242.0, 243.0, 244.0, 245.0, 246.0,
            247.0, 248.0, 249.0, 250.0, 251.0, 252.0, 253.0, 254.0, 255.0, 496.0, 497.0, 498.0,
            499.0, 500.0, 501.0, 502.0, 503.0, 504.0, 505.0, 506.0, 507.0, 508.0, 509.0, 510.0,
            511.0,
        ];
        assert_equals::<R>(out.handle, expected, device);
    }

    /// Exported test
    pub fn cmma_write_output_second_warp_test<R: Runtime>(device: &R::Device) {
        let k = 16;
        let n = 64;
        let out = range_tensor::<R>(k, n, device);
        let acc_sm = range_tensor::<R>(64, 64, device);
        let cube_dim = CubeDim::new(32, 2, 1);
        let cube_count: CubeCount<R::Server> = CubeCount::Static(1, 1, 1);

        let config = CmmaConfig {
            block_size_m: UInt::new(64),
            block_size_k: UInt::new(32),
            block_size_n: UInt::new(64),
            check_m_bounds: false,
            check_k_bounds: false,
            check_n_bounds: false,
            tile_size: UInt::new(16),
            sm_vec: UInt::new(4),
            lhs_transposed: false,
            rhs_transposed: false,
            unroll: false,
        };

        write_output_test::launch::<F32, R>(
            R::client(device),
            cube_count,
            cube_dim,
            TensorArg::vectorized(4, &out.handle, &out.strides, &out.shape),
            ArrayArg::new(&acc_sm.handle, 64 * 64),
            ScalarArg::new(k as u32),
            ScalarArg::new(n as u32),
            config,
        );

        let expected = &[
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            256.0, 257.0, 258.0, 259.0, 260.0, 261.0, 262.0, 263.0, 264.0, 265.0, 266.0, 267.0,
            268.0, 269.0, 270.0, 271.0, 512.0, 513.0, 514.0, 515.0, 516.0, 517.0, 518.0, 519.0,
            520.0, 521.0, 522.0, 523.0, 524.0, 525.0, 526.0, 527.0, 768.0, 769.0, 770.0, 771.0,
            772.0, 773.0, 774.0, 775.0, 776.0, 777.0, 778.0, 779.0, 780.0, 781.0, 782.0, 783.0,
            16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
            30.0, 31.0, 272.0, 273.0, 274.0, 275.0, 276.0, 277.0, 278.0, 279.0, 280.0, 281.0,
            282.0, 283.0, 284.0, 285.0, 286.0, 287.0, 528.0, 529.0, 530.0, 531.0, 532.0, 533.0,
            534.0, 535.0, 536.0, 537.0, 538.0, 539.0, 540.0, 541.0, 542.0, 543.0, 784.0, 785.0,
            786.0, 787.0, 788.0, 789.0, 790.0, 791.0, 792.0, 793.0, 794.0, 795.0, 796.0, 797.0,
            798.0, 799.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0,
            44.0, 45.0, 46.0, 47.0, 288.0, 289.0, 290.0, 291.0, 292.0, 293.0, 294.0, 295.0, 296.0,
            297.0, 298.0, 299.0, 300.0, 301.0, 302.0, 303.0, 544.0, 545.0, 546.0, 547.0, 548.0,
            549.0, 550.0, 551.0, 552.0, 553.0, 554.0, 555.0, 556.0, 557.0, 558.0, 559.0, 800.0,
            801.0, 802.0, 803.0, 804.0, 805.0, 806.0, 807.0, 808.0, 809.0, 810.0, 811.0, 812.0,
            813.0, 814.0, 815.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0,
            59.0, 60.0, 61.0, 62.0, 63.0, 304.0, 305.0, 306.0, 307.0, 308.0, 309.0, 310.0, 311.0,
            312.0, 313.0, 314.0, 315.0, 316.0, 317.0, 318.0, 319.0, 560.0, 561.0, 562.0, 563.0,
            564.0, 565.0, 566.0, 567.0, 568.0, 569.0, 570.0, 571.0, 572.0, 573.0, 574.0, 575.0,
            816.0, 817.0, 818.0, 819.0, 820.0, 821.0, 822.0, 823.0, 824.0, 825.0, 826.0, 827.0,
            828.0, 829.0, 830.0, 831.0, 64.0, 65.0, 66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0, 73.0,
            74.0, 75.0, 76.0, 77.0, 78.0, 79.0, 320.0, 321.0, 322.0, 323.0, 324.0, 325.0, 326.0,
            327.0, 328.0, 329.0, 330.0, 331.0, 332.0, 333.0, 334.0, 335.0, 576.0, 577.0, 578.0,
            579.0, 580.0, 581.0, 582.0, 583.0, 584.0, 585.0, 586.0, 587.0, 588.0, 589.0, 590.0,
            591.0, 832.0, 833.0, 834.0, 835.0, 836.0, 837.0, 838.0, 839.0, 840.0, 841.0, 842.0,
            843.0, 844.0, 845.0, 846.0, 847.0, 80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0,
            88.0, 89.0, 90.0, 91.0, 92.0, 93.0, 94.0, 95.0, 336.0, 337.0, 338.0, 339.0, 340.0,
            341.0, 342.0, 343.0, 344.0, 345.0, 346.0, 347.0, 348.0, 349.0, 350.0, 351.0, 592.0,
            593.0, 594.0, 595.0, 596.0, 597.0, 598.0, 599.0, 600.0, 601.0, 602.0, 603.0, 604.0,
            605.0, 606.0, 607.0, 848.0, 849.0, 850.0, 851.0, 852.0, 853.0, 854.0, 855.0, 856.0,
            857.0, 858.0, 859.0, 860.0, 861.0, 862.0, 863.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0,
            102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 352.0, 353.0,
            354.0, 355.0, 356.0, 357.0, 358.0, 359.0, 360.0, 361.0, 362.0, 363.0, 364.0, 365.0,
            366.0, 367.0, 608.0, 609.0, 610.0, 611.0, 612.0, 613.0, 614.0, 615.0, 616.0, 617.0,
            618.0, 619.0, 620.0, 621.0, 622.0, 623.0, 864.0, 865.0, 866.0, 867.0, 868.0, 869.0,
            870.0, 871.0, 872.0, 873.0, 874.0, 875.0, 876.0, 877.0, 878.0, 879.0, 112.0, 113.0,
            114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0, 124.0, 125.0,
            126.0, 127.0, 368.0, 369.0, 370.0, 371.0, 372.0, 373.0, 374.0, 375.0, 376.0, 377.0,
            378.0, 379.0, 380.0, 381.0, 382.0, 383.0, 624.0, 625.0, 626.0, 627.0, 628.0, 629.0,
            630.0, 631.0, 632.0, 633.0, 634.0, 635.0, 636.0, 637.0, 638.0, 639.0, 880.0, 881.0,
            882.0, 883.0, 884.0, 885.0, 886.0, 887.0, 888.0, 889.0, 890.0, 891.0, 892.0, 893.0,
            894.0, 895.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0, 136.0, 137.0,
            138.0, 139.0, 140.0, 141.0, 142.0, 143.0, 384.0, 385.0, 386.0, 387.0, 388.0, 389.0,
            390.0, 391.0, 392.0, 393.0, 394.0, 395.0, 396.0, 397.0, 398.0, 399.0, 640.0, 641.0,
            642.0, 643.0, 644.0, 645.0, 646.0, 647.0, 648.0, 649.0, 650.0, 651.0, 652.0, 653.0,
            654.0, 655.0, 896.0, 897.0, 898.0, 899.0, 900.0, 901.0, 902.0, 903.0, 904.0, 905.0,
            906.0, 907.0, 908.0, 909.0, 910.0, 911.0, 144.0, 145.0, 146.0, 147.0, 148.0, 149.0,
            150.0, 151.0, 152.0, 153.0, 154.0, 155.0, 156.0, 157.0, 158.0, 159.0, 400.0, 401.0,
            402.0, 403.0, 404.0, 405.0, 406.0, 407.0, 408.0, 409.0, 410.0, 411.0, 412.0, 413.0,
            414.0, 415.0, 656.0, 657.0, 658.0, 659.0, 660.0, 661.0, 662.0, 663.0, 664.0, 665.0,
            666.0, 667.0, 668.0, 669.0, 670.0, 671.0, 912.0, 913.0, 914.0, 915.0, 916.0, 917.0,
            918.0, 919.0, 920.0, 921.0, 922.0, 923.0, 924.0, 925.0, 926.0, 927.0, 160.0, 161.0,
            162.0, 163.0, 164.0, 165.0, 166.0, 167.0, 168.0, 169.0, 170.0, 171.0, 172.0, 173.0,
            174.0, 175.0, 416.0, 417.0, 418.0, 419.0, 420.0, 421.0, 422.0, 423.0, 424.0, 425.0,
            426.0, 427.0, 428.0, 429.0, 430.0, 431.0, 672.0, 673.0, 674.0, 675.0, 676.0, 677.0,
            678.0, 679.0, 680.0, 681.0, 682.0, 683.0, 684.0, 685.0, 686.0, 687.0, 928.0, 929.0,
            930.0, 931.0, 932.0, 933.0, 934.0, 935.0, 936.0, 937.0, 938.0, 939.0, 940.0, 941.0,
            942.0, 943.0, 176.0, 177.0, 178.0, 179.0, 180.0, 181.0, 182.0, 183.0, 184.0, 185.0,
            186.0, 187.0, 188.0, 189.0, 190.0, 191.0, 432.0, 433.0, 434.0, 435.0, 436.0, 437.0,
            438.0, 439.0, 440.0, 441.0, 442.0, 443.0, 444.0, 445.0, 446.0, 447.0, 688.0, 689.0,
            690.0, 691.0, 692.0, 693.0, 694.0, 695.0, 696.0, 697.0, 698.0, 699.0, 700.0, 701.0,
            702.0, 703.0, 944.0, 945.0, 946.0, 947.0, 948.0, 949.0, 950.0, 951.0, 952.0, 953.0,
            954.0, 955.0, 956.0, 957.0, 958.0, 959.0, 192.0, 193.0, 194.0, 195.0, 196.0, 197.0,
            198.0, 199.0, 200.0, 201.0, 202.0, 203.0, 204.0, 205.0, 206.0, 207.0, 448.0, 449.0,
            450.0, 451.0, 452.0, 453.0, 454.0, 455.0, 456.0, 457.0, 458.0, 459.0, 460.0, 461.0,
            462.0, 463.0, 704.0, 705.0, 706.0, 707.0, 708.0, 709.0, 710.0, 711.0, 712.0, 713.0,
            714.0, 715.0, 716.0, 717.0, 718.0, 719.0, 960.0, 961.0, 962.0, 963.0, 964.0, 965.0,
            966.0, 967.0, 968.0, 969.0, 970.0, 971.0, 972.0, 973.0, 974.0, 975.0, 208.0, 209.0,
            210.0, 211.0, 212.0, 213.0, 214.0, 215.0, 216.0, 217.0, 218.0, 219.0, 220.0, 221.0,
            222.0, 223.0, 464.0, 465.0, 466.0, 467.0, 468.0, 469.0, 470.0, 471.0, 472.0, 473.0,
            474.0, 475.0, 476.0, 477.0, 478.0, 479.0, 720.0, 721.0, 722.0, 723.0, 724.0, 725.0,
            726.0, 727.0, 728.0, 729.0, 730.0, 731.0, 732.0, 733.0, 734.0, 735.0, 976.0, 977.0,
            978.0, 979.0, 980.0, 981.0, 982.0, 983.0, 984.0, 985.0, 986.0, 987.0, 988.0, 989.0,
            990.0, 991.0, 224.0, 225.0, 226.0, 227.0, 228.0, 229.0, 230.0, 231.0, 232.0, 233.0,
            234.0, 235.0, 236.0, 237.0, 238.0, 239.0, 480.0, 481.0, 482.0, 483.0, 484.0, 485.0,
            486.0, 487.0, 488.0, 489.0, 490.0, 491.0, 492.0, 493.0, 494.0, 495.0, 736.0, 737.0,
            738.0, 739.0, 740.0, 741.0, 742.0, 743.0, 744.0, 745.0, 746.0, 747.0, 748.0, 749.0,
            750.0, 751.0, 992.0, 993.0, 994.0, 995.0, 996.0, 997.0, 998.0, 999.0, 1000.0, 1001.0,
            1002.0, 1003.0, 1004.0, 1005.0, 1006.0, 1007.0, 240.0, 241.0, 242.0, 243.0, 244.0,
            245.0, 246.0, 247.0, 248.0, 249.0, 250.0, 251.0, 252.0, 253.0, 254.0, 255.0, 496.0,
            497.0, 498.0, 499.0, 500.0, 501.0, 502.0, 503.0, 504.0, 505.0, 506.0, 507.0, 508.0,
            509.0, 510.0, 511.0, 752.0, 753.0, 754.0, 755.0, 756.0, 757.0, 758.0, 759.0, 760.0,
            761.0, 762.0, 763.0, 764.0, 765.0, 766.0, 767.0, 1008.0, 1009.0, 1010.0, 1011.0,
            1012.0, 1013.0, 1014.0, 1015.0, 1016.0, 1017.0, 1018.0, 1019.0, 1020.0, 1021.0, 1022.0,
            1023.0,
        ];
        assert_equals::<R>(out.handle, expected, device);
    }

    /// Exported test
    pub fn cmma_write_output_third_fourth_warps_test<R: Runtime>(device: &R::Device) {
        let k = 32;
        let n = 64;
        let out = range_tensor::<R>(k, n, device);
        let acc_sm = range_tensor::<R>(64, 64, device);
        let cube_dim = CubeDim::new(32, 4, 1);
        let cube_count: CubeCount<R::Server> = CubeCount::Static(1, 1, 1);

        let config = CmmaConfig {
            block_size_m: UInt::new(64),
            block_size_k: UInt::new(32),
            block_size_n: UInt::new(64),
            check_m_bounds: false,
            check_k_bounds: false,
            check_n_bounds: false,
            tile_size: UInt::new(16),
            sm_vec: UInt::new(4),
            lhs_transposed: false,
            rhs_transposed: false,
            unroll: false,
        };

        write_output_test::launch::<F32, R>(
            R::client(device),
            cube_count,
            cube_dim,
            TensorArg::vectorized(4, &out.handle, &out.strides, &out.shape),
            ArrayArg::new(&acc_sm.handle, 64 * 64),
            ScalarArg::new(k as u32),
            ScalarArg::new(n as u32),
            config,
        );

        let expected = &[
            1024., 1025., 1026., 1027., 1028., 1029., 1030., 1031., 1032., 1033., 1034., 1035.,
            1036., 1037., 1038., 1039., 1280., 1281., 1282., 1283., 1284., 1285., 1286., 1287.,
            1288., 1289., 1290., 1291., 1292., 1293., 1294., 1295., 1536., 1537., 1538., 1539.,
            1540., 1541., 1542., 1543., 1544., 1545., 1546., 1547., 1548., 1549., 1550., 1551.,
            1792., 1793., 1794., 1795., 1796., 1797., 1798., 1799., 1800., 1801., 1802., 1803.,
            1804., 1805., 1806., 1807., 1040., 1041., 1042., 1043., 1044., 1045., 1046., 1047.,
            1048., 1049., 1050., 1051., 1052., 1053., 1054., 1055., 1296., 1297., 1298., 1299.,
            1300., 1301., 1302., 1303., 1304., 1305., 1306., 1307., 1308., 1309., 1310., 1311.,
            1552., 1553., 1554., 1555., 1556., 1557., 1558., 1559., 1560., 1561., 1562., 1563.,
            1564., 1565., 1566., 1567., 1808., 1809., 1810., 1811., 1812., 1813., 1814., 1815.,
            1816., 1817., 1818., 1819., 1820., 1821., 1822., 1823., 1056., 1057., 1058., 1059.,
            1060., 1061., 1062., 1063., 1064., 1065., 1066., 1067., 1068., 1069., 1070., 1071.,
            1312., 1313., 1314., 1315., 1316., 1317., 1318., 1319., 1320., 1321., 1322., 1323.,
            1324., 1325., 1326., 1327., 1568., 1569., 1570., 1571., 1572., 1573., 1574., 1575.,
            1576., 1577., 1578., 1579., 1580., 1581., 1582., 1583., 1824., 1825., 1826., 1827.,
            1828., 1829., 1830., 1831., 1832., 1833., 1834., 1835., 1836., 1837., 1838., 1839.,
            1072., 1073., 1074., 1075., 1076., 1077., 1078., 1079., 1080., 1081., 1082., 1083.,
            1084., 1085., 1086., 1087., 1328., 1329., 1330., 1331., 1332., 1333., 1334., 1335.,
            1336., 1337., 1338., 1339., 1340., 1341., 1342., 1343., 1584., 1585., 1586., 1587.,
            1588., 1589., 1590., 1591., 1592., 1593., 1594., 1595., 1596., 1597., 1598., 1599.,
            1840., 1841., 1842., 1843., 1844., 1845., 1846., 1847., 1848., 1849., 1850., 1851.,
            1852., 1853., 1854., 1855., 1088., 1089., 1090., 1091., 1092., 1093., 1094., 1095.,
            1096., 1097., 1098., 1099., 1100., 1101., 1102., 1103., 1344., 1345., 1346., 1347.,
            1348., 1349., 1350., 1351., 1352., 1353., 1354., 1355., 1356., 1357., 1358., 1359.,
            1600., 1601., 1602., 1603., 1604., 1605., 1606., 1607., 1608., 1609., 1610., 1611.,
            1612., 1613., 1614., 1615., 1856., 1857., 1858., 1859., 1860., 1861., 1862., 1863.,
            1864., 1865., 1866., 1867., 1868., 1869., 1870., 1871., 1104., 1105., 1106., 1107.,
            1108., 1109., 1110., 1111., 1112., 1113., 1114., 1115., 1116., 1117., 1118., 1119.,
            1360., 1361., 1362., 1363., 1364., 1365., 1366., 1367., 1368., 1369., 1370., 1371.,
            1372., 1373., 1374., 1375., 1616., 1617., 1618., 1619., 1620., 1621., 1622., 1623.,
            1624., 1625., 1626., 1627., 1628., 1629., 1630., 1631., 1872., 1873., 1874., 1875.,
            1876., 1877., 1878., 1879., 1880., 1881., 1882., 1883., 1884., 1885., 1886., 1887.,
            1120., 1121., 1122., 1123., 1124., 1125., 1126., 1127., 1128., 1129., 1130., 1131.,
            1132., 1133., 1134., 1135., 1376., 1377., 1378., 1379., 1380., 1381., 1382., 1383.,
            1384., 1385., 1386., 1387., 1388., 1389., 1390., 1391., 1632., 1633., 1634., 1635.,
            1636., 1637., 1638., 1639., 1640., 1641., 1642., 1643., 1644., 1645., 1646., 1647.,
            1888., 1889., 1890., 1891., 1892., 1893., 1894., 1895., 1896., 1897., 1898., 1899.,
            1900., 1901., 1902., 1903., 1136., 1137., 1138., 1139., 1140., 1141., 1142., 1143.,
            1144., 1145., 1146., 1147., 1148., 1149., 1150., 1151., 1392., 1393., 1394., 1395.,
            1396., 1397., 1398., 1399., 1400., 1401., 1402., 1403., 1404., 1405., 1406., 1407.,
            1648., 1649., 1650., 1651., 1652., 1653., 1654., 1655., 1656., 1657., 1658., 1659.,
            1660., 1661., 1662., 1663., 1904., 1905., 1906., 1907., 1908., 1909., 1910., 1911.,
            1912., 1913., 1914., 1915., 1916., 1917., 1918., 1919., 1152., 1153., 1154., 1155.,
            1156., 1157., 1158., 1159., 1160., 1161., 1162., 1163., 1164., 1165., 1166., 1167.,
            1408., 1409., 1410., 1411., 1412., 1413., 1414., 1415., 1416., 1417., 1418., 1419.,
            1420., 1421., 1422., 1423., 1664., 1665., 1666., 1667., 1668., 1669., 1670., 1671.,
            1672., 1673., 1674., 1675., 1676., 1677., 1678., 1679., 1920., 1921., 1922., 1923.,
            1924., 1925., 1926., 1927., 1928., 1929., 1930., 1931., 1932., 1933., 1934., 1935.,
            1168., 1169., 1170., 1171., 1172., 1173., 1174., 1175., 1176., 1177., 1178., 1179.,
            1180., 1181., 1182., 1183., 1424., 1425., 1426., 1427., 1428., 1429., 1430., 1431.,
            1432., 1433., 1434., 1435., 1436., 1437., 1438., 1439., 1680., 1681., 1682., 1683.,
            1684., 1685., 1686., 1687., 1688., 1689., 1690., 1691., 1692., 1693., 1694., 1695.,
            1936., 1937., 1938., 1939., 1940., 1941., 1942., 1943., 1944., 1945., 1946., 1947.,
            1948., 1949., 1950., 1951., 1184., 1185., 1186., 1187., 1188., 1189., 1190., 1191.,
            1192., 1193., 1194., 1195., 1196., 1197., 1198., 1199., 1440., 1441., 1442., 1443.,
            1444., 1445., 1446., 1447., 1448., 1449., 1450., 1451., 1452., 1453., 1454., 1455.,
            1696., 1697., 1698., 1699., 1700., 1701., 1702., 1703., 1704., 1705., 1706., 1707.,
            1708., 1709., 1710., 1711., 1952., 1953., 1954., 1955., 1956., 1957., 1958., 1959.,
            1960., 1961., 1962., 1963., 1964., 1965., 1966., 1967., 1200., 1201., 1202., 1203.,
            1204., 1205., 1206., 1207., 1208., 1209., 1210., 1211., 1212., 1213., 1214., 1215.,
            1456., 1457., 1458., 1459., 1460., 1461., 1462., 1463., 1464., 1465., 1466., 1467.,
            1468., 1469., 1470., 1471., 1712., 1713., 1714., 1715., 1716., 1717., 1718., 1719.,
            1720., 1721., 1722., 1723., 1724., 1725., 1726., 1727., 1968., 1969., 1970., 1971.,
            1972., 1973., 1974., 1975., 1976., 1977., 1978., 1979., 1980., 1981., 1982., 1983.,
            1216., 1217., 1218., 1219., 1220., 1221., 1222., 1223., 1224., 1225., 1226., 1227.,
            1228., 1229., 1230., 1231., 1472., 1473., 1474., 1475., 1476., 1477., 1478., 1479.,
            1480., 1481., 1482., 1483., 1484., 1485., 1486., 1487., 1728., 1729., 1730., 1731.,
            1732., 1733., 1734., 1735., 1736., 1737., 1738., 1739., 1740., 1741., 1742., 1743.,
            1984., 1985., 1986., 1987., 1988., 1989., 1990., 1991., 1992., 1993., 1994., 1995.,
            1996., 1997., 1998., 1999., 1232., 1233., 1234., 1235., 1236., 1237., 1238., 1239.,
            1240., 1241., 1242., 1243., 1244., 1245., 1246., 1247., 1488., 1489., 1490., 1491.,
            1492., 1493., 1494., 1495., 1496., 1497., 1498., 1499., 1500., 1501., 1502., 1503.,
            1744., 1745., 1746., 1747., 1748., 1749., 1750., 1751., 1752., 1753., 1754., 1755.,
            1756., 1757., 1758., 1759., 2000., 2001., 2002., 2003., 2004., 2005., 2006., 2007.,
            2008., 2009., 2010., 2011., 2012., 2013., 2014., 2015., 1248., 1249., 1250., 1251.,
            1252., 1253., 1254., 1255., 1256., 1257., 1258., 1259., 1260., 1261., 1262., 1263.,
            1504., 1505., 1506., 1507., 1508., 1509., 1510., 1511., 1512., 1513., 1514., 1515.,
            1516., 1517., 1518., 1519., 1760., 1761., 1762., 1763., 1764., 1765., 1766., 1767.,
            1768., 1769., 1770., 1771., 1772., 1773., 1774., 1775., 2016., 2017., 2018., 2019.,
            2020., 2021., 2022., 2023., 2024., 2025., 2026., 2027., 2028., 2029., 2030., 2031.,
            1264., 1265., 1266., 1267., 1268., 1269., 1270., 1271., 1272., 1273., 1274., 1275.,
            1276., 1277., 1278., 1279., 1520., 1521., 1522., 1523., 1524., 1525., 1526., 1527.,
            1528., 1529., 1530., 1531., 1532., 1533., 1534., 1535., 1776., 1777., 1778., 1779.,
            1780., 1781., 1782., 1783., 1784., 1785., 1786., 1787., 1788., 1789., 1790., 1791.,
            2032., 2033., 2034., 2035., 2036., 2037., 2038., 2039., 2040., 2041., 2042., 2043.,
            2044., 2045., 2046., 2047.,
        ];
        assert_equals_range::<R>(out.handle, expected, 1024..2048, device);
    }
}
