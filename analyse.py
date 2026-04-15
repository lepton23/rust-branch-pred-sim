#!/usr/bin/env python3

import os
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

RESULTS_DIR = "results"
PLOTS_DIR = os.path.join(RESULTS_DIR, "plots")
os.makedirs(PLOTS_DIR, exist_ok=True)

sns.set_theme(style="whitegrid")
PALETTE = "viridis"


def annotate_grouped_bars(ax, fontsize=6, rotation=90, padding=0.15):
    for patch in ax.patches:
        height = patch.get_height()
        if height <= 0:
            continue
        ax.text(
            patch.get_x() + patch.get_width() / 2,
            height + padding,
            f"{height:.4g}",
            ha="center", va="bottom",
            fontsize=fontsize, rotation=rotation,
        )


def annotate_stacked_bars(ax, min_pct=1.5, fontsize=7):
    for patch in ax.patches:
        height = patch.get_height()
        if height < min_pct:
            continue
        x = patch.get_x() + patch.get_width() / 2
        y = patch.get_y() + height / 2
        ax.text(x, y, f"{height:.4g}%", ha="center", va="center",
                fontsize=fontsize, color="white" if height > 8 else "black")


# ---------------------------------------------------------------------------
# Load data
# ---------------------------------------------------------------------------

always = pd.read_csv(os.path.join(RESULTS_DIR, "results_always.csv"))
bias = pd.read_csv(os.path.join(RESULTS_DIR, "bias_data.csv"))

# Each predictor has two counter variants:
#   predictor_name  → (csv stem,          predictor label, counter label)
PREDICTOR_FILES = [
    ("results_twobit",          "Two-bit",  "Basic"),
    ("results_gshare",          "GShare",   "Basic"),
    ("results_profiled",        "Profiled", "Basic"),
    ("results_twobit_sat",      "Two-bit",  "Saturating"),
    ("results_gshare_sat",      "GShare",   "Saturating"),
    ("results_profiled_sat",    "Profiled", "Saturating"),
]

# Rename the rate column for convenience
RATE_COL = "mispredict_rate_%"

frames = []
for stem, pred_label, counter_label in PREDICTOR_FILES:
    path = os.path.join(RESULTS_DIR, f"{stem}.csv")
    df = pd.read_csv(path)
    df["predictor"] = pred_label
    df["counter"] = counter_label
    frames.append(df)

combined = pd.concat(frames, ignore_index=True)

# Tidy up trace names (strip .out suffix)
for df in (always, combined, bias):
    df["trace"] = df["trace"].str.replace(r"\.out$", "", regex=True)

TABLE_SIZES = sorted(combined["table_size"].unique())
PREDICTORS = ["Two-bit", "GShare", "Profiled"]
COUNTERS = ["Basic", "Saturating"]


# ---------------------------------------------------------------------------
# 1. Mispredict rate vs table size
#    One figure per predictor; rows = counter variants, columns = traces
# ---------------------------------------------------------------------------

for pred in PREDICTORS:
    pred_data = combined[combined["predictor"] == pred]
    fig, axes = plt.subplots(
        len(COUNTERS), 1, figsize=(10, 7), sharey=True, sharex=True
    )
    for ax, counter in zip(axes, COUNTERS):
        data = pred_data[pred_data["counter"] == counter]
        sns.lineplot(
            data=data,
            x="table_size",
            y=RATE_COL,
            hue="trace",
            marker="o",
            palette="Set1",
            ax=ax,
        )
        ax.set_title(f"{counter} counter")
        ax.set_xscale("log", base=2)
        ax.set_xticks(TABLE_SIZES)
        ax.set_xticklabels(TABLE_SIZES)
        ax.set_xlabel("Table size (entries)")
        ax.set_ylabel("Mispredict rate (%)")
        ax.get_legend().remove()

    handles, labels = axes[-1].get_legend_handles_labels()
    fig.legend(handles, labels, title="Trace", bbox_to_anchor=(1.01, 0.5), loc="center left")
    fig.suptitle(f"{pred} – mispredict rate vs table size", fontsize=13)
    fig.tight_layout()
    fname = f"mispredict_vs_table_size_{pred.lower().replace('-', '')}.png"
    fig.savefig(os.path.join(PLOTS_DIR, fname), dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"Saved: {fname}")


# ---------------------------------------------------------------------------
# 2. Predictor comparison – grouped bar chart at each table size
#    Hue = predictor+counter combination
# ---------------------------------------------------------------------------

combined["pred_counter"] = combined["predictor"] + "\n(" + combined["counter"] + ")"
pred_counter_order = [
    f"{p}\n({c})" for p in PREDICTORS for c in COUNTERS
]

for size in TABLE_SIZES:
    subset = combined[combined["table_size"] == size]
    fig, ax = plt.subplots(figsize=(13, 5))
    sns.barplot(
        data=subset,
        x="trace",
        y=RATE_COL,
        hue="pred_counter",
        hue_order=pred_counter_order,
        palette=PALETTE,
        ax=ax,
    )
    annotate_grouped_bars(ax)
    ax.set_title(f"Predictor comparison – table size {size}")
    ax.set_xlabel("Trace")
    ax.set_ylabel("Mispredict rate (%)")
    ax.legend(title="Predictor (counter)", bbox_to_anchor=(1.01, 0.5), loc="center left",
              fontsize="x-small")
    fig.tight_layout()
    fname = f"predictor_comparison_size{size}.png"
    fig.savefig(os.path.join(PLOTS_DIR, fname), dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"Saved: {fname}")


# ---------------------------------------------------------------------------
# 4. Branch bias distribution
# ---------------------------------------------------------------------------

bin_edges = [0.0, 0.01, 0.3, 0.45, 0.55, 0.7, 0.99, 1.0]
bin_labels = ["0–1%", "1–30%", "30–45%", "45–55%", "55–70%", "70–99%", "99–100%"]
bin_colors = ["#d62728", "#ff7f0e", "#bcbd22", "#17becf", "#1f77b4", "#997fb4", "#bb3fc3"]

bias["bias_bin"] = pd.cut(
    bias["taken_pct"],
    bins=bin_edges,
    labels=bin_labels,
    include_lowest=True,
)

bin_counts = (
    bias.groupby(["trace", "bias_bin"], observed=True)
    .size()
    .unstack(fill_value=0)
)
bin_pct = bin_counts.div(bin_counts.sum(axis=1), axis=0) * 100

fig, ax = plt.subplots(figsize=(9, 5))
bin_pct[bin_labels].plot(
    kind="bar",
    stacked=True,
    color=bin_colors,
    ax=ax,
    edgecolor="white",
    width=0.6,
)
annotate_stacked_bars(ax)
ax.set_title("Branch bias distribution per trace")
ax.set_xlabel("Trace")
ax.set_ylabel("Percentage of branches (%)")
ax.set_xticklabels(ax.get_xticklabels(), rotation=30, ha="right")
ax.legend(title="Taken rate", loc="lower right", fontsize="x-small")
fig.tight_layout()
fig.savefig(os.path.join(PLOTS_DIR, "branch_bias_distribution.png"), dpi=150)
plt.close(fig)
print("Saved: branch_bias_distribution.png")


# ---------------------------------------------------------------------------
# 5. Easy vs hard branch breakdown per trace (stacked bar)
# ---------------------------------------------------------------------------

counts = bias.groupby(["trace", "category"]).size().unstack(fill_value=0)
counts_pct = counts.div(counts.sum(axis=1), axis=0) * 100

fig, ax = plt.subplots(figsize=(9, 5))
counts_pct.plot(
    kind="bar",
    stacked=True,
    color=sns.color_palette(PALETTE, n_colors=2),
    ax=ax,
    edgecolor="white",
)
annotate_stacked_bars(ax)
ax.set_title("Proportion of easy vs hard branches per trace")
ax.set_xlabel("Trace")
ax.set_ylabel("Percentage of branches (%)")
ax.set_xticklabels(ax.get_xticklabels(), rotation=30, ha="right")
ax.legend(title="Category", loc="lower right")
fig.tight_layout()
fig.savefig(os.path.join(PLOTS_DIR, "easy_vs_hard_branches.png"), dpi=150)
plt.close(fig)
print("Saved: easy_vs_hard_branches.png")


# ---------------------------------------------------------------------------
# 6. Heatmap – colour = % change in mispredict rate relative to smallest table
#              annotation = actual mispredict rate
#    One heatmap per (predictor, counter) pair
# ---------------------------------------------------------------------------

for pred in PREDICTORS:
    for counter in COUNTERS:
        data = (
            combined[(combined["predictor"] == pred) & (combined["counter"] == counter)]
            .pivot(index="trace", columns="table_size", values=RATE_COL)
        )
        # Relative % change from the smallest table size (first column)
        baseline = data.iloc[:, 0]
        rel_change = data.sub(baseline, axis=0).div(baseline, axis=0) * 100

        # Annotations show actual mispredict rate
        annot = data.map(lambda v: f"{v:.2f}%")

        abs_max = rel_change.abs().max().max()
        fig, ax = plt.subplots(figsize=(7, 5))
        sns.heatmap(
            rel_change,
            annot=annot,
            fmt="",
            cmap="rocket",
            center=0,
            vmin=-abs_max,
            vmax=abs_max,
            linewidths=0.5,
            ax=ax,
            cbar_kws={"label": "% change from smallest table size"},
        )
        ax.set_title(
            f"{pred} ({counter} counter)\n"
            f"mispredict rate (colour = relative change from size {TABLE_SIZES[0]})"
        )
        ax.set_xlabel("Table size")
        ax.set_ylabel("Trace")
        fig.tight_layout()
        counter_slug = counter.lower().replace("-", "")
        pred_slug = pred.lower().replace("-", "")
        fname = f"heatmap_{pred_slug}_{counter_slug}.png"
        fig.savefig(os.path.join(PLOTS_DIR, fname), dpi=150)
        plt.close(fig)
        print(f"Saved: {fname}")


print("\nAll plots written to", PLOTS_DIR)
