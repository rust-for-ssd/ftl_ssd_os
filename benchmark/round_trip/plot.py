import numpy as np
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
from matplotlib.ticker import ScalarFormatter
import scipy.stats as stats
import seaborn as sns
import pandas as pd
import os

# Create plots directory if it doesn't exist
os.makedirs('plots', exist_ok=True)

# Set up the seaborn theme for professional publication
sns.set_theme(style="whitegrid", context="paper")
plt.rcParams.update({
    'font.family': 'serif',
    'font.serif': ['Computer Modern Roman'],
    'axes.labelsize': 11,
    'axes.titlesize': 12,
    'xtick.labelsize': 10,
    'ytick.labelsize': 10,
    'legend.fontsize': 10,
    'figure.figsize': (10, 6),
    'figure.dpi': 300,
})

# Color palette
palette = sns.color_palette("colorblind")

# Read data from files - excluding clang
with open('benchmark/round_trip/times/gcc.txt', 'r') as f:
    gcc_data = np.array([float(line.strip()) for line in f if line.strip()])

with open('benchmark/round_trip/times/rust.txt', 'r') as f:
    rust_data = np.array([float(line.strip()) for line in f if line.strip()])

# Create dataframes for easier plotting with seaborn
max_length = max(len(gcc_data), len(rust_data))
df_time_series = pd.DataFrame({
    'Time (s)': np.arange(1, max_length + 1)
})

# Add GCC and Rust data
df_time_series['GCC'] = pd.Series(gcc_data)
df_time_series['Rust'] = pd.Series(rust_data)

# Prepare data for box plot
df_combined = pd.DataFrame({
    'Compiler': ['GCC']*len(gcc_data) + ['Rust']*len(rust_data),
    'Throughput (stages/s)': np.concatenate([gcc_data, rust_data])
})

# Calculate statistics for each compiler
def calculate_stats(data):
    return {
        'mean': np.mean(data),
        'median': np.median(data),
        'std': np.std(data),
        'min': np.min(data),
        'max': np.max(data),
        'ci_lower': stats.t.interval(0.95, len(data)-1, loc=np.mean(data), scale=stats.sem(data))[0],
        'ci_upper': stats.t.interval(0.95, len(data)-1, loc=np.mean(data), scale=stats.sem(data))[1]
    }

gcc_stats = calculate_stats(gcc_data)
rust_stats = calculate_stats(rust_data)

# Create figure with two subplots
fig, (ax1, ax2) = plt.subplots(2, 1, height_ratios=[3, 1], figsize=(10, 8))

# Time series plot (top)
sns.lineplot(
    data=pd.melt(
        df_time_series, 
        id_vars=['Time (s)'], 
        value_vars=['GCC', 'Rust'],
        var_name='Program',
        value_name='Throughput'
    ),
    x='Time (s)',
    y='Throughput',
    hue='Program',
    palette={'GCC': palette[1], 'Rust': palette[2]},
    ax=ax1,
    linewidth=1.2,
)

# Add mean lines
ax1.axhline(y=gcc_stats['mean'], color=palette[1], linestyle='--', alpha=0.7, linewidth=1)
ax1.axhline(y=rust_stats['mean'], color=palette[2], linestyle='--', alpha=0.7, linewidth=1)

# Customize top plot
ax1.set_title('Sustained Throughput', fontweight='bold')
ax1.set_ylabel('Throughput (stages/s)')

# Format y-axis to use scientific notation properly
formatter = ScalarFormatter(useMathText=True)
formatter.set_scientific(True)
formatter.set_powerlimits((-3, 4))
ax1.yaxis.set_major_formatter(formatter)

# Box plot (bottom)
sns.boxplot(
    data=df_combined,
    x='Compiler',
    y='Throughput (stages/s)',
    order=['GCC', 'Rust'],  # Explicit category order
    palette=[palette[1], palette[2]],
    ax=ax2,
    width=0.6,
    showmeans=True,
    meanprops={"marker":"o", "markerfacecolor":"white", "markeredgecolor":"black", "markersize":6},
    showfliers=True,
    flierprops={'marker':'o', 'markerfacecolor':'none', 'markeredgecolor':'gray', 'markersize':4},
    notch=True,
)

# Format y-axis on box plot to use scientific notation
ax2.yaxis.set_major_formatter(formatter)
ax2.set_xlabel('Compiler')
ax2.set_ylabel('Throughput (stages/s)')

# Calculate speedup percentage
gcc_to_rust = ((rust_stats['mean'] / gcc_stats['mean']) - 1) * 100

# Tight layout and save figures
plt.tight_layout()
plt.savefig('plots/gcc_rust_throughput_comparison.pdf', dpi=300, bbox_inches='tight')
plt.savefig('plots/gcc_rust_throughput_comparison.png', dpi=300, bbox_inches='tight')

# Save statistics to a file
with open('plots/compiler_stats.txt', 'w') as f:
    f.write("Detailed Statistics:\n")
    f.write("-" * 50 + "\n")
    f.write(f"GCC:   mean={gcc_stats['mean']:.2f}, median={gcc_stats['median']:.2f}, std={gcc_stats['std']:.2f}\n")
    f.write(f"      min={gcc_stats['min']:.2f}, max={gcc_stats['max']:.2f}\n")
    f.write(f"      95% CI: [{gcc_stats['ci_lower']:.2f}, {gcc_stats['ci_upper']:.2f}]\n")
    f.write("-" * 50 + "\n")
    f.write(f"Rust:  mean={rust_stats['mean']:.2f}, median={rust_stats['median']:.2f}, std={rust_stats['std']:.2f}\n")
    f.write(f"      min={rust_stats['min']:.2f}, max={rust_stats['max']:.2f}\n")
    f.write(f"      95% CI: [{rust_stats['ci_lower']:.2f}, {rust_stats['ci_upper']:.2f}]\n")
    f.write("-" * 50 + "\n\n")
    f.write("Performance Comparison:\n")
    f.write("-" * 50 + "\n")
    f.write(f"Rust vs GCC:   {rust_stats['mean']/gcc_stats['mean']:.2f}x  ({gcc_to_rust:+.1f}%)\n")

# Print to console as well
print("\nDetailed Statistics saved to plots/compiler_stats.txt")
print(f"Plots saved to plots/gcc_rust_throughput_comparison.pdf and plots/gcc_rust_throughput_comparison.png")

plt.show()