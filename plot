#!/usr/bin/python3

import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

def create_arg_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('file')
    parser.add_argument('-l', '--lambdas', nargs="+", default=[0.0, 0.25, 0.5, 0.75, 1.0])
    parser.add_argument('--max', action='store_true')

    return parser


def get_data(filename):
    data = pd.read_csv(filename)
    data = data.round(3)
    data['arr_cr'] = data['arr'] / data['opt']
    data['two_stage_cr'] = data['two_stage'] / data['opt']
    return data





def plot_eta(df, args):
    df = df[df['lambda'].isin(args.lambdas)]

    df_arr = df.loc[:, ['lambda', 'sigma', 'arr_cr']]
    df_ts = df.loc[:, ['lambda', 'sigma', 'two_stage_cr']]

    grouped_data = df_arr.groupby(['lambda', 'sigma']).mean().unstack('lambda')
    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(
            style='D-', markersize=4, linewidth=1.2, label=f"ARR (λ = {l:1.2f})", legend=True)

    grouped_data = df_ts.groupby(['lambda', 'sigma']).mean().unstack('lambda')
    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(
            style='o--', markersize=4, linewidth=1.2, label=f"2-S (λ = {l:1.2f})", legend=True)

    #plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel('sigma')
    plt.ylabel('Empirical competitive ratio')
    plt.legend()
    plt.tight_layout()
    #plt.axis([0, max_bin, 0.99, 1.1])


    fig = plt.gcf()
    fig.set_dpi(500)
    fig.set_size_inches(4,2.5)
    # fig.subplots_adjust(right=0.7)
    #fig.savefig("result.png", dpi=400)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.file):

        data = get_data(parsed_args.file)
        plot_eta(data, parsed_args)
       # plot_lambda(data, float(parsed_args.bin_size),
       #             parsed_args, det_alg, pred_alg)
        plt.show()
    else:
        print("Path not valid!")
