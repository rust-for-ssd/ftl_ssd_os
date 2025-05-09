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

# Read data from the connector_per_component folder
# data_folder = 'benchmark/sustained_throughput/connector_per_component'
data_folder = 'benchmark/sustained_throughput/pipeline_per_cmd'

with open(os.path.join(data_folder, 'write.txt'), 'r') as f:
    write_data = np.array([float(line.strip()) for line in f if line.strip()])

with open(os.path.join(data_folder, 'read.txt'), 'r') as f:
    read_data = np.array([float(line.strip()) for line in f if line.strip()])

with open(os.path.join(data_folder, 'mixed.txt'), 'r') as f:
    mixed_data = np.array([float(line.strip()) for line in f if line.strip()])

# Create dataframes for easier plotting with seaborn
max_length = max(len(write_data), len(read_data), len(mixed_data))
df_time_series = pd.DataFrame({
    'Time (s)': np.arange(1, max_length + 1)
})

# Add data for each operation type
df_time_series['Write'] = pd.Series(write_data)
df_time_series['Read'] = pd.Series(read_data)
df_time_series['Mixed'] = pd.Series(mixed_data)

# Prepare data for box plot
df_combined = pd.DataFrame({
    'Operation': ['Write']*len(write_data) + ['Read']*len(read_data) + ['Mixed']*len(mixed_data),
    'Throughput (reqs/s)': np.concatenate([write_data, read_data, mixed_data])
})

# Calculate statistics for each operation type
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

write_stats = calculate_stats(write_data)
read_stats = calculate_stats(read_data)
mixed_stats = calculate_stats(mixed_data)

# Create figure with two subplots
fig, (ax1, ax2) = plt.subplots(2, 1, height_ratios=[3, 1], figsize=(10, 8))

# Time series plot (top)
sns.lineplot(
    data=pd.melt(
        df_time_series, 
        id_vars=['Time (s)'], 
        value_vars=['Write', 'Read', 'Mixed'],
        var_name='Operation',
        value_name='Throughput'
    ),
    x='Time (s)',
    y='Throughput',
    hue='Operation',
    palette={'Write': palette[0], 'Read': palette[1], 'Mixed': palette[2]},
    ax=ax1,
    linewidth=1.2,
)

# Add mean lines
ax1.axhline(y=write_stats['mean'], color=palette[0], linestyle='--', alpha=0.7, linewidth=1)
ax1.axhline(y=read_stats['mean'], color=palette[1], linestyle='--', alpha=0.7, linewidth=1)
ax1.axhline(y=mixed_stats['mean'], color=palette[2], linestyle='--', alpha=0.7, linewidth=1)

# Customize top plot
# ax1.set_title('Operation Throughput Comparison', fontweight='bold')
ax1.set_ylabel('Throughput (reqs/s)')

# Format y-axis to use scientific notation properly
formatter = ScalarFormatter(useMathText=True)
formatter.set_scientific(True)
formatter.set_powerlimits((-3, 4))
ax1.yaxis.set_major_formatter(formatter)

# Box plot (bottom)
sns.boxplot(
    data=df_combined,
    x='Operation',
    y='Throughput (reqs/s)',
    order=['Write', 'Read', 'Mixed'],  # Explicit category order
    palette=[palette[0], palette[1], palette[2]],
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
ax2.set_xlabel('Operation Type')
ax2.set_ylabel('Throughput (reqs/s)')

# Calculate performance comparisons
write_to_read = ((write_stats['mean'] / read_stats['mean']) - 1) * 100
write_to_mixed = ((write_stats['mean'] / mixed_stats['mean']) - 1) * 100
read_to_mixed = ((read_stats['mean'] / mixed_stats['mean']) - 1) * 100

# Tight layout and save figures
plt.tight_layout()
plt.savefig('plots/operation_throughput_comparison.pdf', dpi=300, bbox_inches='tight')
plt.savefig('plots/operation_throughput_comparison.png', dpi=300, bbox_inches='tight')

# Save statistics to a file
with open('plots/operation_stats.txt', 'w') as f:
    f.write("Detailed Statistics:\n")
    f.write("-" * 50 + "\n")
    f.write(f"Write: mean={write_stats['mean']:.2f}, median={write_stats['median']:.2f}, std={write_stats['std']:.2f}\n")
    f.write(f"      min={write_stats['min']:.2f}, max={write_stats['max']:.2f}\n")
    f.write(f"      95% CI: [{write_stats['ci_lower']:.2f}, {write_stats['ci_upper']:.2f}]\n")
    f.write("-" * 50 + "\n")
    f.write(f"Read:  mean={read_stats['mean']:.2f}, median={read_stats['median']:.2f}, std={read_stats['std']:.2f}\n")
    f.write(f"      min={read_stats['min']:.2f}, max={read_stats['max']:.2f}\n")
    f.write(f"      95% CI: [{read_stats['ci_lower']:.2f}, {read_stats['ci_upper']:.2f}]\n")
    f.write("-" * 50 + "\n")
    f.write(f"Mixed: mean={mixed_stats['mean']:.2f}, median={mixed_stats['median']:.2f}, std={mixed_stats['std']:.2f}\n")
    f.write(f"      min={mixed_stats['min']:.2f}, max={mixed_stats['max']:.2f}\n")
    f.write(f"      95% CI: [{mixed_stats['ci_lower']:.2f}, {mixed_stats['ci_upper']:.2f}]\n")
    f.write("-" * 50 + "\n\n")
    
    f.write("Performance Comparisons:\n")
    f.write("-" * 50 + "\n")
    f.write(f"Write vs Read:  {write_stats['mean']/read_stats['mean']:.2f}x  ({write_to_read:+.1f}%)\n")
    f.write(f"Write vs Mixed: {write_stats['mean']/mixed_stats['mean']:.2f}x  ({write_to_mixed:+.1f}%)\n")
    f.write(f"Read vs Mixed:  {read_stats['mean']/mixed_stats['mean']:.2f}x  ({read_to_mixed:+.1f}%)\n")

# Print to console as well
print("\nDetailed Statistics saved to plots/operation_stats.txt")
print(f"Plots saved to plots/operation_throughput_comparison.pdf and plots/operation_throughput_comparison.png")

# plt.show()