# Round Trip Time Analysis from File
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
import os

# Create a folder for plots if it doesn't exist
plot_folder = 'benchmark/plots'
os.makedirs(plot_folder, exist_ok=True)

# Read data from bench.txt file
try:
    # Attempt to read the file with rtt data (one value per line)
    with open('benchmark/bench.txt', 'r') as file:
        # Read all lines after finding the marker
        lines = file.readlines()
        
        # Find the line containing the marker and extract lines after it
        start_index = None
        for i, line in enumerate(lines):
            if '--- ssd_os is READY! ---' in line:
                start_index = i + 1  # Start after the marker
                break
        
        if start_index is None:
            raise ValueError("Marker '--- ssd_os is READY! ---' not found in file.")
        
        # Now extract data from the following lines
        input = []
        for line in lines[start_index:]:
            if line.strip():  # Avoid empty lines
                input.append(int(line.strip()))
    
    # Convert from 10MHz clock cycles to milliseconds
    # 1 cycle at 10MHz = 0.1 μs = 0.0001 ms
    # conversion_factor = 0.0001  # 10MHz clock cycle to milliseconds
    rtt_data = input
    
    print(f"Successfully loaded {len(rtt_data)} data points from bench.txt")
    print(f"Converted clock cycles at 10MHz to milliseconds (1 cycle = 0.0001 ms)")
except FileNotFoundError:
    print("Error: bench.txt file not found. Please ensure the file exists in the current directory.")
    # Provide sample data in case file is not found
    rtt_data = []
except ValueError as ve:
    print(ve)
    rtt_data = []
except Exception as e:
    print(f"Error reading bench.txt: {e}")
    rtt_data = []

# Proceed only if we have data
if rtt_data:
    # Convert to pandas Series
    rtt_series = pd.Series(rtt_data)
    # Calculate statistics
    mean_rtt = rtt_series.mean()
    median_rtt = rtt_series.median()
    std_rtt = rtt_series.std()
    min_rtt = rtt_series.min()
    max_rtt = rtt_series.max()
    # Display statistics
    print(f"\nStatistics for Round Trip Time (RTT):")
    print(f"Mean: {mean_rtt:.4f} ms")
    print(f"Median: {median_rtt:.4f} ms")
    print(f"Standard Deviation: {std_rtt:.4f} ms")
    print(f"Minimum: {min_rtt:.4f} ms")
    print(f"Maximum: {max_rtt:.4f} ms")
    
    # Create and save individual plots
    
    # Plot 1: Histogram with KDE
    plt.figure(figsize=(10, 6))
    sns.histplot(rtt_series, kde=True, color='skyblue')
    plt.axvline(mean_rtt, color='red', linestyle='--', label=f'Mean: {mean_rtt:.4f}')
    plt.axvline(median_rtt, color='green', linestyle='-.', label=f'Median: {median_rtt:.4f}')
    plt.title('RTT Distribution')
    plt.xlabel('Round Trip Time (ms)')
    plt.ylabel('Frequency')
    plt.legend()
    plt.tight_layout()
    plt.savefig(f'{plot_folder}/rtt_histogram.png', dpi=300)
    plt.close()
    
    # Plot 2: Box plot
    plt.figure(figsize=(8, 6))
    sns.boxplot(y=rtt_series, color='lightgreen')
    plt.title('RTT Box Plot')
    plt.ylabel('Round Trip Time (ms)')
    plt.tight_layout()
    plt.savefig(f'{plot_folder}/rtt_boxplot.png', dpi=300)
    plt.close()
    
    # Plot 3: Line plot of raw data
    plt.figure(figsize=(12, 6))
    plt.plot(rtt_series, marker='o', linestyle='-', alpha=0.7)
    plt.axhline(mean_rtt, color='red', linestyle='--', label=f'Mean: {mean_rtt:.4f}')
    plt.fill_between(range(len(rtt_series)), 
                     mean_rtt - std_rtt, 
                     mean_rtt + std_rtt, 
                     alpha=0.2, 
                     color='red', 
                     label=f'±1 Std Dev: {std_rtt:.4f}')
    plt.title('RTT Values Over Samples')
    plt.xlabel('Sample Index')
    plt.ylabel('Round Trip Time (ms)')
    plt.legend()
    plt.tight_layout()
    plt.savefig(f'{plot_folder}/rtt_lineplot.png', dpi=300)
    plt.close()
    
    # Create combined plot for display
    plt.figure(figsize=(16, 10))
    # Plot 1: Histogram with KDE
    plt.subplot(2, 2, 1)
    sns.histplot(rtt_series, kde=True, color='skyblue')
    plt.axvline(mean_rtt, color='red', linestyle='--', label=f'Mean: {mean_rtt:.4f}')
    plt.axvline(median_rtt, color='green', linestyle='-.', label=f'Median: {median_rtt:.4f}')
    plt.title('RTT Distribution')
    plt.xlabel('Round Trip Time (ms)')
    plt.ylabel('Frequency')
    plt.legend()
    
    # Plot 2: Box plot
    plt.subplot(2, 2, 2)
    sns.boxplot(y=rtt_series, color='lightgreen')
    plt.title('RTT Box Plot')
    plt.ylabel('Round Trip Time (ms)')
    
    # Plot 3: Line plot of raw data
    plt.subplot(2, 1, 2)
    plt.plot(rtt_series, marker='o', linestyle='-', alpha=0.7)
    plt.axhline(mean_rtt, color='red', linestyle='--', label=f'Mean: {mean_rtt:.4f}')
    plt.fill_between(range(len(rtt_series)), 
                     mean_rtt - std_rtt, 
                     mean_rtt + std_rtt, 
                     alpha=0.2, 
                     color='red', 
                     label=f'±1 Std Dev: {std_rtt:.4f}')
    plt.title('RTT Values Over Samples')
    plt.xlabel('Sample Index')
    plt.ylabel('Round Trip Time (ms)')
    plt.legend()
    
    # plt.tight_layout()
    plt.savefig(f'{plot_folder}/rtt_combined.png', dpi=300)
    # plt.show()
    
    # Create a DataFrame for further analysis if needed
    rtt_df = pd.DataFrame({
        'RTT (ms)': rtt_series,
        'Sample': range(1, len(rtt_series) + 1)
    })
    # Show a summary of the data
    # display(rtt_df.describe())
    
    # Save statistics to a text file
    with open(f'{plot_folder}/rtt_stats.txt', 'w') as stats_file:
        stats_file.write("Statistics for Round Trip Time (RTT):\n")
        stats_file.write(f"Mean: {mean_rtt:.4f} ms\n")
        stats_file.write(f"Median: {median_rtt:.4f} ms\n")
        stats_file.write(f"Standard Deviation: {std_rtt:.4f} ms\n")
        stats_file.write(f"Minimum: {min_rtt:.4f} ms\n")
        stats_file.write(f"Maximum: {max_rtt:.4f} ms\n")
    
    print(f"\nAnalysis complete. Results saved to {plot_folder}/")
    print(f"Generated files:")
    print(f"- {plot_folder}/rtt_histogram.png")
    print(f"- {plot_folder}/rtt_boxplot.png")
    print(f"- {plot_folder}/rtt_lineplot.png")
    print(f"- {plot_folder}/rtt_combined.png")
    print(f"- {plot_folder}/rtt_stats.txt")
else:
    print("No data available for analysis. Please check the bench.txt file.")