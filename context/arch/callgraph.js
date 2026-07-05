/* ============================================================
   callgraph.js - written by upkeep-context callgraph_scan.py
   schema: cg1
   Script-owned file: re-runs regenerate it wholesale.
   ============================================================ */
window.CALLGRAPH = JSON.parse(`{
  "schema": "cg1",
  "lang": "rust",
  "scope": "entry: main · rust · 31 files · 135 functions",
  "stats": [
    [
      "functions",
      "135",
      "",
      "in scope"
    ],
    [
      "call edges",
      "477",
      "",
      "static"
    ],
    [
      "resolved",
      "272",
      "ok",
      "57%"
    ],
    [
      "ambiguous",
      "0",
      "warn",
      "0%"
    ],
    [
      "external",
      "205",
      "dim",
      "collapsed"
    ],
    [
      "dynamic",
      "0",
      "violet",
      "0%"
    ]
  ],
  "legend": [
    [
      "resolved",
      "unique static target"
    ],
    [
      "trait",
      "via trait/interface method"
    ],
    [
      "ambiguous",
      "n candidate targets"
    ],
    [
      "dynamic",
      "call site kept · target unknown"
    ],
    [
      "external",
      "outside the analysed source"
    ]
  ],
  "typesLabel": "types in scope",
  "types": [
    "Action",
    "App",
    "Bridge",
    "CoinbaseConfig",
    "CounterSet",
    "CounterSnapshot",
    "EngineState",
    "ErrorSeverity"
  ],
  "nodes": [
    {
      "id": "src_main_rs_main_21",
      "name": "main()",
      "meta": "src/main.rs:21",
      "cert": "resolved",
      "row": 0,
      "entry": true
    },
    {
      "id": "src_ui_app_rs_run_762",
      "name": "run()",
      "meta": "src/ui/app.rs:762",
      "cert": "resolved",
      "row": 1
    },
    {
      "id": "src_main_rs_run_live_coinbase_69",
      "name": "run_live_coinbase()",
      "meta": "src/main.rs:69",
      "cert": "resolved",
      "row": 1
    },
    {
      "id": "src_ui_app_rs_install_panic_hook_923",
      "name": "install_panic_hook()",
      "meta": "src/ui/app.rs:923",
      "cert": "resolved",
      "row": 2
    },
    {
      "id": "src_ui_app_rs_run_loop_844",
      "name": "run_loop()",
      "meta": "src/ui/app.rs:844",
      "cert": "resolved",
      "row": 2
    },
    {
      "id": "src_ui_app_rs_run_with_app_784",
      "name": "run_with_app()",
      "meta": "src/ui/app.rs:784",
      "cert": "resolved",
      "row": 2
    },
    {
      "id": "src_telemetry_writer_rs_spawn_writer_88",
      "name": "spawn_writer()",
      "meta": "src/telemetry/writer.rs:88",
      "cert": "resolved",
      "row": 2
    },
    {
      "id": "src_ui_panes_rs_render_22",
      "name": "render()",
      "meta": "src/ui/panes.rs:22",
      "cert": "resolved",
      "row": 3
    },
    {
      "id": "src_ui_theme_rs_block_bar_56",
      "name": "block_bar()",
      "meta": "src/ui/theme.rs:56",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×3 sites"
    },
    {
      "id": "src_ui_theme_rs_bold_35",
      "name": "bold()",
      "meta": "src/ui/theme.rs:35",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×5 sites"
    },
    {
      "id": "src_ui_theme_rs_fg_40",
      "name": "fg()",
      "meta": "src/ui/theme.rs:40",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×7 sites"
    },
    {
      "id": "src_ui_theme_rs_fg_bold_45",
      "name": "fg_bold()",
      "meta": "src/ui/theme.rs:45",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_theme_rs_fg_dim_50",
      "name": "fg_dim()",
      "meta": "src/ui/theme.rs:50",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×9 sites"
    },
    {
      "id": "src_ui_panes_rs_format_price_cents_858",
      "name": "format_price_cents()",
      "meta": "src/ui/panes.rs:858",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×5 sites"
    },
    {
      "id": "src_ui_panes_rs_level_line_287",
      "name": "level_line()",
      "meta": "src/ui/panes.rs:287",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_theme_rs_neutral_25",
      "name": "neutral()",
      "meta": "src/ui/theme.rs:25",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×7 sites"
    },
    {
      "id": "src_ui_panes_rs_pane_block_850",
      "name": "pane_block()",
      "meta": "src/ui/panes.rs:850",
      "cert": "resolved",
      "row": 4,
      "badge": "◇ ×6 sites"
    },
    {
      "id": "src_ui_panes_rs_render_body_152",
      "name": "render_body()",
      "meta": "src/ui/panes.rs:152",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_depth_of_book_185",
      "name": "render_depth_of_book()",
      "meta": "src/ui/panes.rs:185",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_engine_summary_636",
      "name": "render_engine_summary()",
      "meta": "src/ui/panes.rs:636",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_keybinds_132",
      "name": "render_keybinds()",
      "meta": "src/ui/panes.rs:132",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_latency_353",
      "name": "render_latency()",
      "meta": "src/ui/panes.rs:353",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_mid_chart_516",
      "name": "render_mid_chart()",
      "meta": "src/ui/panes.rs:516",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_throughput_574",
      "name": "render_throughput()",
      "meta": "src/ui/panes.rs:574",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_top_status_37",
      "name": "render_top_status()",
      "meta": "src/ui/panes.rs:37",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "src_ui_panes_rs_render_trade_tape_299",
      "name": "render_trade_tape()",
      "meta": "src/ui/panes.rs:299",
      "cert": "resolved",
      "row": 4
    },
    {
      "id": "ext_core_index",
      "name": "core::index",
      "meta": "external",
      "cert": "external",
      "ext": true,
      "row": 0,
      "doc": "Outside the analysed source; 38 call sites reach it."
    },
    {
      "id": "ext_core_poll",
      "name": "core::poll",
      "meta": "external",
      "cert": "external",
      "ext": true,
      "row": 0,
      "doc": "Outside the analysed source; 32 call sites reach it."
    },
    {
      "id": "ext_core_max",
      "name": "core::max",
      "meta": "external",
      "cert": "external",
      "ext": true,
      "row": 0,
      "doc": "Outside the analysed source; 20 call sites reach it."
    }
  ],
  "edges": [
    [
      "src_main_rs_main_21",
      "src_main_rs_run_live_coinbase_69",
      "resolved"
    ],
    [
      "src_main_rs_main_21",
      "src_ui_app_rs_run_762",
      "resolved"
    ],
    [
      "src_main_rs_run_live_coinbase_69",
      "src_telemetry_writer_rs_spawn_writer_88",
      "resolved"
    ],
    [
      "src_main_rs_run_live_coinbase_69",
      "src_ui_app_rs_run_with_app_784",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_762",
      "src_telemetry_writer_rs_spawn_writer_88",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_762",
      "src_ui_app_rs_install_panic_hook_923",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_762",
      "src_ui_app_rs_run_loop_844",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_loop_844",
      "src_ui_panes_rs_render_22",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_with_app_784",
      "src_ui_app_rs_install_panic_hook_923",
      "resolved"
    ],
    [
      "src_ui_app_rs_run_with_app_784",
      "src_ui_app_rs_run_loop_844",
      "resolved"
    ],
    [
      "src_ui_panes_rs_level_line_287",
      "src_ui_panes_rs_format_price_cents_858",
      "resolved"
    ],
    [
      "src_ui_panes_rs_level_line_287",
      "src_ui_theme_rs_block_bar_56",
      "resolved"
    ],
    [
      "src_ui_panes_rs_level_line_287",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_level_line_287",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_pane_block_850",
      "src_ui_theme_rs_bold_35",
      "resolved"
    ],
    [
      "src_ui_panes_rs_pane_block_850",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_22",
      "src_ui_panes_rs_render_body_152",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_22",
      "src_ui_panes_rs_render_keybinds_132",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_22",
      "src_ui_panes_rs_render_top_status_37",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_depth_of_book_185",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_engine_summary_636",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_latency_353",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_mid_chart_516",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_throughput_574",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_body_152",
      "src_ui_panes_rs_render_trade_tape_299",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_panes_rs_format_price_cents_858",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_panes_rs_level_line_287",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_depth_of_book_185",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_panes_rs_format_price_cents_858",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_theme_rs_block_bar_56",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_theme_rs_bold_35",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_engine_summary_636",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_keybinds_132",
      "src_ui_theme_rs_fg_bold_45",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_keybinds_132",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_latency_353",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_latency_353",
      "src_ui_theme_rs_bold_35",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_latency_353",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_latency_353",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_latency_353",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_mid_chart_516",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_mid_chart_516",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_throughput_574",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_throughput_574",
      "src_ui_theme_rs_bold_35",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_throughput_574",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_throughput_574",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_throughput_574",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_panes_rs_format_price_cents_858",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_theme_rs_bold_35",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_theme_rs_fg_bold_45",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_top_status_37",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_panes_rs_format_price_cents_858",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_panes_rs_pane_block_850",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_theme_rs_block_bar_56",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_theme_rs_fg_40",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_theme_rs_fg_dim_50",
      "resolved"
    ],
    [
      "src_ui_panes_rs_render_trade_tape_299",
      "src_ui_theme_rs_neutral_25",
      "resolved"
    ]
  ],
  "tree": [
    {
      "pre": "",
      "tog": "▾",
      "name": "main()",
      "meta": ":21",
      "hot": true
    },
    {
      "pre": "├─ ",
      "tog": "▸",
      "name": "parse_live()",
      "meta": ":58"
    },
    {
      "pre": "├─ ",
      "tog": "▸",
      "name": "parse_seed()",
      "meta": ":44"
    },
    {
      "pre": "├─ ",
      "tog": "▸",
      "name": "run_headless()",
      "meta": ":142"
    },
    {
      "pre": "├─ ",
      "tog": "▾",
      "name": "run_live_coinbase()",
      "meta": ":69"
    },
    {
      "pre": "│  ├─ ",
      "tog": "▾",
      "name": "run_coinbase()",
      "meta": ":124"
    },
    {
      "pre": "│  │  └─ ",
      "tog": "▸",
      "name": "connect_and_pump()",
      "meta": ":154"
    },
    {
      "pre": "│  ├─ ",
      "tog": "▾",
      "name": "spawn_writer()",
      "meta": ":88"
    },
    {
      "pre": "│  │  ├─ ",
      "tog": "▸",
      "name": "resolve_path()",
      "meta": ":112"
    },
    {
      "pre": "│  │  └─ ",
      "tog": "▸",
      "name": "writer_loop()",
      "meta": ":121"
    },
    {
      "pre": "│  └─ ",
      "tog": "▾",
      "name": "run_with_app()",
      "meta": ":784",
      "hot": true
    },
    {
      "pre": "│     ├─ ",
      "tog": "▸",
      "name": "install_panic_hook()",
      "meta": ":923"
    },
    {
      "pre": "│     ├─ ",
      "tog": "▸",
      "name": "restore_terminal()",
      "meta": ":899"
    },
    {
      "pre": "│     ├─ ",
      "tog": "▸",
      "name": "run_loop()",
      "meta": ":844"
    },
    {
      "pre": "│     └─ ",
      "tog": "▸",
      "name": "setup_terminal()",
      "meta": ":890"
    },
    {
      "pre": "└─ ",
      "tog": "▾",
      "name": "run()",
      "meta": ":762",
      "hot": true
    }
  ],
  "fns": [
    {
      "id": "src_book_market_rs_buy_100",
      "name": "buy",
      "file": "src/book/market.rs",
      "line": 100,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_book_market_rs_empty_market_113",
      "name": "empty_market",
      "file": "src/book/market.rs",
      "line": 113,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_book_market_rs_auto_register_on_submit_120",
      "name": "auto_register_on_submit",
      "file": "src/book/market.rs",
      "line": 120,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_market_rs_separate_books_per_symbol_129",
      "name": "separate_books_per_symbol",
      "file": "src/book/market.rs",
      "line": 129,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_order_book_rs_crosses_377",
      "name": "crosses",
      "file": "src/book/order_book.rs",
      "line": 377,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_book_price_level_rs_order_129",
      "name": "order",
      "file": "src/book/price_level.rs",
      "line": 129,
      "in": 6,
      "out": 0
    },
    {
      "id": "src_book_price_level_rs_empty_level_has_zero_tot",
      "name": "empty_level_has_zero_total",
      "file": "src/book/price_level.rs",
      "line": 142,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_book_price_level_rs_push_back_maintains_tota",
      "name": "push_back_maintains_total",
      "file": "src/book/price_level.rs",
      "line": 149,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_push_back_rejects_wrong_",
      "name": "push_back_rejects_wrong_price",
      "file": "src/book/price_level.rs",
      "line": 158,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_pop_front_returns_oldest",
      "name": "pop_front_returns_oldest_first",
      "file": "src/book/price_level.rs",
      "line": 168,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_record_execution_updates",
      "name": "record_execution_updates_total",
      "file": "src/book/price_level.rs",
      "line": 178,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_remove_by_id_returns_and",
      "name": "remove_by_id_returns_and_decrements",
      "file": "src/book/price_level.rs",
      "line": 186,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_remove_by_id_missing_ret",
      "name": "remove_by_id_missing_returns_none",
      "file": "src/book/price_level.rs",
      "line": 201,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_book_price_level_rs_pop_front_empty_returns_",
      "name": "pop_front_empty_returns_none",
      "file": "src/book/price_level.rs",
      "line": 208,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_errors_rs_recoverable_variants_classify_reco",
      "name": "recoverable_variants_classify_recoverable",
      "file": "src/errors.rs",
      "line": 137,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_errors_rs_invariant_violation_is_fatal_176",
      "name": "invariant_violation_is_fatal",
      "file": "src/errors.rs",
      "line": 176,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_errors_rs_errors_format_human_readably_183",
      "name": "errors_format_human_readably",
      "file": "src/errors.rs",
      "line": 183,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_events_fill_rs_ts_51",
      "name": "ts",
      "file": "src/events/fill.rs",
      "line": 51,
      "in": 3,
      "out": 0
    },
    {
      "id": "src_events_fill_rs_rejects_zero_quantity_56",
      "name": "rejects_zero_quantity",
      "file": "src/events/fill.rs",
      "line": 56,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_fill_rs_rejects_self_match_69",
      "name": "rejects_self_match",
      "file": "src/events/fill.rs",
      "line": 69,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_fill_rs_accepts_valid_fill_76",
      "name": "accepts_valid_fill",
      "file": "src/events/fill.rs",
      "line": 76,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_lifecycle_rs_ts_149",
      "name": "ts",
      "file": "src/events/lifecycle.rs",
      "line": 149,
      "in": 5,
      "out": 0
    },
    {
      "id": "src_events_lifecycle_rs_placed_rejects_zero_quan",
      "name": "placed_rejects_zero_quantity",
      "file": "src/events/lifecycle.rs",
      "line": 154,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_lifecycle_rs_filled_rejects_zero_exec",
      "name": "filled_rejects_zero_executed",
      "file": "src/events/lifecycle.rs",
      "line": 167,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_lifecycle_rs_cancelled_is_infallible_",
      "name": "cancelled_is_infallible",
      "file": "src/events/lifecycle.rs",
      "line": 173,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_lifecycle_rs_rejected_carries_reason_",
      "name": "rejected_carries_reason",
      "file": "src/events/lifecycle.rs",
      "line": 181,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_lifecycle_rs_accessor_methods_work_ac",
      "name": "accessor_methods_work_across_variants",
      "file": "src/events/lifecycle.rs",
      "line": 196,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_events_quote_rs_live_rejects_zero_quantity_7",
      "name": "live_rejects_zero_quantity",
      "file": "src/events/quote.rs",
      "line": 79,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_events_quote_rs_cleared_carries_zero_quantit",
      "name": "cleared_carries_zero_quantity",
      "file": "src/events/quote.rs",
      "line": 91,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_events_quote_rs_side_conversion_99",
      "name": "side_conversion",
      "file": "src/events/quote.rs",
      "line": 99,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_feed_bridge_rs_px_200",
      "name": "px",
      "file": "src/feed/bridge.rs",
      "line": 200,
      "in": 4,
      "out": 0
    },
    {
      "id": "src_feed_bridge_rs_qty_203",
      "name": "qty",
      "file": "src/feed/bridge.rs",
      "line": 203,
      "in": 4,
      "out": 0
    },
    {
      "id": "src_feed_bridge_rs_snapshot_routes_to_correct_id",
      "name": "snapshot_routes_to_correct_idx",
      "file": "src/feed/bridge.rs",
      "line": 208,
      "in": 0,
      "out": 2
    },
    {
      "id": "src_feed_bridge_rs_update_with_nonzero_emits_can",
      "name": "update_with_nonzero_emits_cancel_then_submit",
      "file": "src/feed/bridge.rs",
      "line": 222,
      "in": 0,
      "out": 2
    },
    {
      "id": "src_feed_bridge_rs_update_with_zero_emits_cancel",
      "name": "update_with_zero_emits_cancel_only",
      "file": "src/feed/bridge.rs",
      "line": 250,
      "in": 0,
      "out": 2
    },
    {
      "id": "src_feed_bridge_rs_unknown_symbol_drops_silently",
      "name": "unknown_symbol_drops_silently",
      "file": "src/feed/bridge.rs",
      "line": 272,
      "in": 0,
      "out": 2
    },
    {
      "id": "src_feed_coinbase_rs_run_coinbase_124",
      "name": "run_coinbase",
      "file": "src/feed/coinbase.rs",
      "line": 124,
      "in": 1,
      "out": 1
    },
    {
      "id": "src_feed_coinbase_rs_connect_and_pump_154",
      "name": "connect_and_pump",
      "file": "src/feed/coinbase.rs",
      "line": 154,
      "in": 1,
      "out": 1
    },
    {
      "id": "src_feed_coinbase_rs_handle_text_200",
      "name": "handle_text",
      "file": "src/feed/coinbase.rs",
      "line": 200,
      "in": 1,
      "out": 3
    },
    {
      "id": "src_feed_coinbase_rs_symbol_from_product_id_287",
      "name": "symbol_from_product_id",
      "file": "src/feed/coinbase.rs",
      "line": 287,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_feed_coinbase_rs_parse_price_300",
      "name": "parse_price",
      "file": "src/feed/coinbase.rs",
      "line": 300,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_feed_coinbase_rs_parse_qty_313",
      "name": "parse_qty",
      "file": "src/feed/coinbase.rs",
      "line": 313,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_main_rs_main_21",
      "name": "main",
      "file": "src/main.rs",
      "line": 21,
      "in": 0,
      "out": 5
    },
    {
      "id": "src_main_rs_parse_seed_44",
      "name": "parse_seed",
      "file": "src/main.rs",
      "line": 44,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_main_rs_parse_live_58",
      "name": "parse_live",
      "file": "src/main.rs",
      "line": 58,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_main_rs_run_live_coinbase_69",
      "name": "run_live_coinbase",
      "file": "src/main.rs",
      "line": 69,
      "in": 1,
      "out": 3
    },
    {
      "id": "src_main_rs_run_headless_142",
      "name": "run_headless",
      "file": "src/main.rs",
      "line": 142,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_metrics_windows_rs_record_then_snapshot_with",
      "name": "record_then_snapshot_within_1s",
      "file": "src/metrics/windows.rs",
      "line": 85,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_metrics_windows_rs_empty_snapshot_is_zero_95",
      "name": "empty_snapshot_is_zero",
      "file": "src/metrics/windows.rs",
      "line": 95,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_order_rs_status_str_170",
      "name": "status_str",
      "file": "src/order.rs",
      "line": 170,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_order_rs_px_197",
      "name": "px",
      "file": "src/order.rs",
      "line": 197,
      "in": 9,
      "out": 0
    },
    {
      "id": "src_order_rs_id_200",
      "name": "id",
      "file": "src/order.rs",
      "line": 200,
      "in": 9,
      "out": 0
    },
    {
      "id": "src_order_rs_qty_203",
      "name": "qty",
      "file": "src/order.rs",
      "line": 203,
      "in": 8,
      "out": 0
    },
    {
      "id": "src_order_rs_new_rejects_zero_quantity_208",
      "name": "new_rejects_zero_quantity",
      "file": "src/order.rs",
      "line": 208,
      "in": 0,
      "out": 2
    },
    {
      "id": "src_order_rs_fill_partial_then_full_214",
      "name": "fill_partial_then_full",
      "file": "src/order.rs",
      "line": 214,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_fill_rejects_overfill_without_mutat",
      "name": "fill_rejects_overfill_without_mutation",
      "file": "src/order.rs",
      "line": 227,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_fill_rejects_zero_amount_237",
      "name": "fill_rejects_zero_amount",
      "file": "src/order.rs",
      "line": 237,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_fill_rejected_after_terminal_244",
      "name": "fill_rejected_after_terminal",
      "file": "src/order.rs",
      "line": 244,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_cancel_open_order_253",
      "name": "cancel_open_order",
      "file": "src/order.rs",
      "line": 253,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_cancel_partial_order_260",
      "name": "cancel_partial_order",
      "file": "src/order.rs",
      "line": 260,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_cancel_rejected_after_terminal_269",
      "name": "cancel_rejected_after_terminal",
      "file": "src/order.rs",
      "line": 269,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_order_rs_observing_state_does_not_move_order",
      "name": "observing_state_does_not_move_order",
      "file": "src/order.rs",
      "line": 276,
      "in": 0,
      "out": 3
    },
    {
      "id": "src_simulator_market_rs_standard_normal_248",
      "name": "standard_normal",
      "file": "src/simulator/market.rs",
      "line": 248,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_poisson_sample_257",
      "name": "poisson_sample",
      "file": "src/simulator/market.rs",
      "line": 257,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_sample_distance_275",
      "name": "sample_distance",
      "file": "src/simulator/market.rs",
      "line": 275,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_deterministic_under_fixe",
      "name": "deterministic_under_fixed_seed",
      "file": "src/simulator/market.rs",
      "line": 297,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_step_emits_orders_within",
      "name": "step_emits_orders_within_expected_band",
      "file": "src/simulator/market.rs",
      "line": 319,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_mid_price_stays_in_reaso",
      "name": "mid_price_stays_in_reasonable_neighbourhood",
      "file": "src/simulator/market.rs",
      "line": 334,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_simulator_market_rs_pathological_dt_never_di",
      "name": "pathological_dt_never_diverges_the_mid",
      "file": "src/simulator/market.rs",
      "line": 346,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_telemetry_writer_rs_spawn_writer_88",
      "name": "spawn_writer",
      "file": "src/telemetry/writer.rs",
      "line": 88,
      "in": 2,
      "out": 2
    },
    {
      "id": "src_telemetry_writer_rs_resolve_path_112",
      "name": "resolve_path",
      "file": "src/telemetry/writer.rs",
      "line": 112,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_telemetry_writer_rs_writer_loop_121",
      "name": "writer_loop",
      "file": "src/telemetry/writer.rs",
      "line": 121,
      "in": 1,
      "out": 1
    },
    {
      "id": "src_telemetry_writer_rs_write_event_158",
      "name": "write_event",
      "file": "src/telemetry/writer.rs",
      "line": 158,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_telemetry_writer_rs_noop_handle_accepts_reco",
      "name": "noop_handle_accepts_records_without_panicking",
      "file": "src/telemetry/writer.rs",
      "line": 174,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_telemetry_writer_rs_handle_clones_cheaply_19",
      "name": "handle_clones_cheaply",
      "file": "src/telemetry/writer.rs",
      "line": 191,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_telemetry_writer_rs_writer_truncates_and_wri",
      "name": "writer_truncates_and_writes",
      "file": "src/telemetry/writer.rs",
      "line": 202,
      "in": 0,
      "out": 1
    },
    {
      "id": "src_types_rs_symbol_round_trip_371",
      "name": "symbol_round_trip",
      "file": "src/types.rs",
      "line": 371,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_symbol_rejects_empty_or_too_long_37",
      "name": "symbol_rejects_empty_or_too_long",
      "file": "src/types.rs",
      "line": 378,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_symbol_lexicographic_ord_386",
      "name": "symbol_lexicographic_ord",
      "file": "src/types.rs",
      "line": 386,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_order_id_rejects_zero_392",
      "name": "order_id_rejects_zero",
      "file": "src/types.rs",
      "line": 392,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_side_opposite_is_involution_401",
      "name": "side_opposite_is_involution",
      "file": "src/types.rs",
      "line": 401,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_px_from_cents_rejects_zero_408",
      "name": "px_from_cents_rejects_zero",
      "file": "src/types.rs",
      "line": 408,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_px_from_dollars_rounds_to_nearest_c",
      "name": "px_from_dollars_rounds_to_nearest_cent",
      "file": "src/types.rs",
      "line": 415,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_px_from_dollars_rejects_invalid_inp",
      "name": "px_from_dollars_rejects_invalid_inputs",
      "file": "src/types.rs",
      "line": 422,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_px_ordering_works_on_cents_431",
      "name": "px_ordering_works_on_cents",
      "file": "src/types.rs",
      "line": 431,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_qty_zero_representable_arithmetic_c",
      "name": "qty_zero_representable_arithmetic_checked",
      "file": "src/types.rs",
      "line": 437,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_ts_now_is_post_2020_445",
      "name": "ts_now_is_post_2020",
      "file": "src/types.rs",
      "line": 445,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_ts_duration_since_is_symmetric_452",
      "name": "ts_duration_since_is_symmetric",
      "file": "src/types.rs",
      "line": 452,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_status_one_way_transitions_460",
      "name": "status_one_way_transitions",
      "file": "src/types.rs",
      "line": 460,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_types_rs_status_classification_475",
      "name": "status_classification",
      "file": "src/types.rs",
      "line": 475,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_map_key_747",
      "name": "map_key",
      "file": "src/ui/app.rs",
      "line": 747,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_run_762",
      "name": "run",
      "file": "src/ui/app.rs",
      "line": 762,
      "in": 1,
      "out": 5
    },
    {
      "id": "src_ui_app_rs_run_with_app_784",
      "name": "run_with_app",
      "file": "src/ui/app.rs",
      "line": 784,
      "in": 1,
      "out": 4
    },
    {
      "id": "src_ui_app_rs_side_str_794",
      "name": "side_str",
      "file": "src/ui/app.rs",
      "line": 794,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_quote_side_str_801",
      "name": "quote_side_str",
      "file": "src/ui/app.rs",
      "line": 801,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_rejection_reason_str_808",
      "name": "rejection_reason_str",
      "file": "src/ui/app.rs",
      "line": 808,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_action_str_818",
      "name": "action_str",
      "file": "src/ui/app.rs",
      "line": 818,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_mode_str_830",
      "name": "mode_str",
      "file": "src/ui/app.rs",
      "line": 830,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_push_ring_837",
      "name": "push_ring",
      "file": "src/ui/app.rs",
      "line": 837,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_run_loop_844",
      "name": "run_loop",
      "file": "src/ui/app.rs",
      "line": 844,
      "in": 2,
      "out": 2
    },
    {
      "id": "src_ui_app_rs_setup_terminal_890",
      "name": "setup_terminal",
      "file": "src/ui/app.rs",
      "line": 890,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_restore_terminal_899",
      "name": "restore_terminal",
      "file": "src/ui/app.rs",
      "line": 899,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_restore_terminal_raw_913",
      "name": "restore_terminal_raw",
      "file": "src/ui/app.rs",
      "line": 913,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_app_rs_install_panic_hook_923",
      "name": "install_panic_hook",
      "file": "src/ui/app.rs",
      "line": 923,
      "in": 2,
      "out": 1
    },
    {
      "id": "src_ui_panes_rs_render_22",
      "name": "render",
      "file": "src/ui/panes.rs",
      "line": 22,
      "in": 1,
      "out": 3
    },
    {
      "id": "src_ui_panes_rs_render_top_status_37",
      "name": "render_top_status",
      "file": "src/ui/panes.rs",
      "line": 37,
      "in": 1,
      "out": 7
    },
    {
      "id": "src_ui_panes_rs_render_keybinds_132",
      "name": "render_keybinds",
      "file": "src/ui/panes.rs",
      "line": 132,
      "in": 1,
      "out": 2
    },
    {
      "id": "src_ui_panes_rs_render_body_152",
      "name": "render_body",
      "file": "src/ui/panes.rs",
      "line": 152,
      "in": 1,
      "out": 6
    },
    {
      "id": "src_ui_panes_rs_render_depth_of_book_185",
      "name": "render_depth_of_book",
      "file": "src/ui/panes.rs",
      "line": 185,
      "in": 1,
      "out": 7
    },
    {
      "id": "src_ui_panes_rs_level_line_287",
      "name": "level_line",
      "file": "src/ui/panes.rs",
      "line": 287,
      "in": 1,
      "out": 4
    },
    {
      "id": "src_ui_panes_rs_render_trade_tape_299",
      "name": "render_trade_tape",
      "file": "src/ui/panes.rs",
      "line": 299,
      "in": 1,
      "out": 7
    },
    {
      "id": "src_ui_panes_rs_render_latency_353",
      "name": "render_latency",
      "file": "src/ui/panes.rs",
      "line": 353,
      "in": 1,
      "out": 8
    },
    {
      "id": "src_ui_panes_rs_budget_for_502",
      "name": "budget_for",
      "file": "src/ui/panes.rs",
      "line": 502,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs_render_mid_chart_516",
      "name": "render_mid_chart",
      "file": "src/ui/panes.rs",
      "line": 516,
      "in": 1,
      "out": 2
    },
    {
      "id": "src_ui_panes_rs_render_throughput_574",
      "name": "render_throughput",
      "file": "src/ui/panes.rs",
      "line": 574,
      "in": 1,
      "out": 6
    },
    {
      "id": "src_ui_panes_rs_render_engine_summary_636",
      "name": "render_engine_summary",
      "file": "src/ui/panes.rs",
      "line": 636,
      "in": 1,
      "out": 11
    },
    {
      "id": "src_ui_panes_rs_ofi_gauge_829",
      "name": "ofi_gauge",
      "file": "src/ui/panes.rs",
      "line": 829,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs_pane_block_850",
      "name": "pane_block",
      "file": "src/ui/panes.rs",
      "line": 850,
      "in": 6,
      "out": 2
    },
    {
      "id": "src_ui_panes_rs_format_price_cents_858",
      "name": "format_price_cents",
      "file": "src/ui/panes.rs",
      "line": 858,
      "in": 5,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs_format_latency_ns_864",
      "name": "format_latency_ns",
      "file": "src/ui/panes.rs",
      "line": 864,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs_format_clock_ns_878",
      "name": "format_clock_ns",
      "file": "src/ui/panes.rs",
      "line": 878,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs_format_duration_887",
      "name": "format_duration",
      "file": "src/ui/panes.rs",
      "line": 887,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_panes_rs__typecheck_902",
      "name": "_typecheck",
      "file": "src/ui/panes.rs",
      "line": 902,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_neutral_25",
      "name": "neutral",
      "file": "src/ui/theme.rs",
      "line": 25,
      "in": 7,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_dim_30",
      "name": "dim",
      "file": "src/ui/theme.rs",
      "line": 30,
      "in": 0,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_bold_35",
      "name": "bold",
      "file": "src/ui/theme.rs",
      "line": 35,
      "in": 5,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_fg_40",
      "name": "fg",
      "file": "src/ui/theme.rs",
      "line": 40,
      "in": 7,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_fg_bold_45",
      "name": "fg_bold",
      "file": "src/ui/theme.rs",
      "line": 45,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_fg_dim_50",
      "name": "fg_dim",
      "file": "src/ui/theme.rs",
      "line": 50,
      "in": 9,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_block_bar_56",
      "name": "block_bar",
      "file": "src/ui/theme.rs",
      "line": 56,
      "in": 3,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_sparkline_80",
      "name": "sparkline",
      "file": "src/ui/theme.rs",
      "line": 80,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_pressure_bar_114",
      "name": "pressure_bar",
      "file": "src/ui/theme.rs",
      "line": 114,
      "in": 2,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_microprice_axis_138",
      "name": "microprice_axis",
      "file": "src/ui/theme.rs",
      "line": 138,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_distribution_bar_166",
      "name": "distribution_bar",
      "file": "src/ui/theme.rs",
      "line": 166,
      "in": 1,
      "out": 0
    },
    {
      "id": "src_ui_theme_rs_twin_progress_209",
      "name": "twin_progress",
      "file": "src/ui/theme.rs",
      "line": 209,
      "in": 1,
      "out": 0
    }
  ]
}`);
