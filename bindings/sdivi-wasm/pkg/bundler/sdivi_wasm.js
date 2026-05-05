/* @ts-self-types="./sdivi_wasm.d.ts" */

import * as wasm from "./sdivi_wasm_bg.wasm";
import { __wbg_set_wasm } from "./sdivi_wasm_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    assemble_snapshot, compute_boundary_violations, compute_change_coupling, compute_coupling_topology, compute_delta, compute_pattern_metrics, compute_thresholds_check, compute_trend, detect_boundaries, infer_boundaries, init_wasm, list_categories, normalize_and_hash
} from "./sdivi_wasm_bg.js";
