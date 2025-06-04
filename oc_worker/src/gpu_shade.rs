pub const SHADER_SOURCE: &str = r#"
    struct SingleU32 {
	    value: u32,
	};

	@group(0) @binding(0)
	var<storage, read> a_t: SingleU32;

	@group(0) @binding(1)
	var<storage, read> b_t: SingleU32;

	@group(0) @binding(2)
	var<storage, read_write> result_t: SingleU32;

	@compute @workgroup_size(1)
	fn add_u32(@builtin(global_invocation_id) id: vec3<u32>) {
	    result_t.value = a_t.value + b_t.value;
	}

    @group(0) @binding(3)
    var<storage, read> a: array<f32>;

    @group(0) @binding(4)
    var<storage, read> b: array<f32>;

    @group(0) @binding(5)
    var<storage, read_write> result: array<f32>;


    @compute @workgroup_size(1)
    fn add(@builtin(global_invocation_id) id: vec3<u32>) {
        let i = id.x;
        result[i] = a[i] + b[i];
    }

    struct MatrixHeader {
        long: u32,
        width: u32,
    };

    @group(0) @binding(6)
    var<storage, read> a_header: MatrixHeader;

    @group(0) @binding(7)
    var<storage, read> b_header: MatrixHeader;

    @group(0) @binding(8)
    var<storage, read> a_data: array<f32>;

    @group(0) @binding(9)
    var<storage, read> b_data: array<f32>;

    @group(0) @binding(10)
    var<storage, read_write> matrix_result: array<f32>;

    const TILE_SIZE: u32 = 16;
    var<workgroup> tileA: array<array<f32, TILE_SIZE>, TILE_SIZE>;
    var<workgroup> tileB: array<array<f32, TILE_SIZE>, TILE_SIZE>;

    @compute @workgroup_size(TILE_SIZE, TILE_SIZE)
    fn matrix_multiply(@builtin(global_invocation_id) gid: vec3<u32>,
                       @builtin(local_invocation_id) lid: vec3<u32>) {
        let row: u32 = gid.y;
        let col: u32 = gid.x;

        let local_row = lid.y;
        let local_col = lid.x;

        let m = a_header.long;
        let k = a_header.width;
        let n = b_header.width;

        var sum: f32 = 0.0;

        for (var t: u32 = 0u; t < (k + TILE_SIZE - 1u) / TILE_SIZE; t = t + 1u) {
            let tiled_col = t * TILE_SIZE + local_col;
            let tiled_row = t * TILE_SIZE + local_row;

            if (tiled_col < k && row < m) {
                tileA[local_row][local_col] = a_data[row * k + tiled_col];
            } else {
                tileA[local_row][local_col] = 0.0;
            }

            if (tiled_row < k && col < n) {
                tileB[local_row][local_col] = b_data[tiled_row * n + col];
            } else {
                tileB[local_row][local_col] = 0.0;
            }

            workgroupBarrier();

            for (var i: u32 = 0u; i < TILE_SIZE; i = i + 1u) {
                sum = sum + tileA[local_row][i] * tileB[i][local_col];
            }

            workgroupBarrier();
        }

        if (row < m && col < n) {
            matrix_result[row * n + col] = sum;
        }
    }

    //vec muti matrix

    @group(0) @binding(11)
    var<storage, read> vmm_vector: array<f32>;

    @group(0) @binding(12)
    var<storage, read> vmm_matrix_header: MatrixHeader;

    @group(0) @binding(13)
    var<storage, read> vmm_matrix: array<f32>;

    @group(0) @binding(14)
    var<storage, read_write> vmm_result: array<f32>;

    @compute @workgroup_size(64)
    fn vector_matrix_multiply(@builtin(global_invocation_id) gid: vec3<u32>) {
        let col = gid.x;

        if (col >= vmm_matrix_header.width) {
            return;
        }

        var sum: f32 = 0.0;
        for (var row: u32 = 0u; row < vmm_matrix_header.long; row = row + 1u) {
            let mat_val = vmm_matrix[col + row * vmm_matrix_header.width];
            let vec_val = vmm_vector[row];
            sum = sum + vec_val * mat_val;
        }

        vmm_result[col] = sum;
    }
"#;