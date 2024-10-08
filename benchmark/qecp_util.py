import json, math, re, math
import numpy as np
from dataclasses import dataclass, field


class RuntimeStatistics:
    """
    read runtime statistics given filename
    """

    def __init__(self, filename, apply_entries=None):
        assert isinstance(filename, str)
        self.global_config = None
        # single simulation; currently we only support a single config
        self.local_config = None
        self.entries = []
        with open(filename, "r", encoding="utf8") as f:
            for line_idx, line in enumerate(f):
                line = line.strip("\r\n ")
                if line == "":
                    break
                if line_idx == 0:
                    assert line.startswith("#f ")
                    self.global_config = json.loads(line[3:])
                elif line_idx == 1:
                    assert line.startswith("# ")
                    self.local_config = json.loads(line[2:])
                else:
                    value = json.loads(line)
                    if apply_entries is None:
                        self.entries.append(value)
                    else:
                        self.entries.append(apply_entries(value))
        assert self.local_config is not None

    def __repr__(self):
        return f"RuntimeStatistics {{ config: {self.global_config}, entries: [...{len(self.entries)}] }}"

    def sum_decoding_time(self):
        decoding_time = 0
        for entry in self.entries:
            decoding_time += entry["elapsed"]["decode"]
        return decoding_time

    def decoding_time_relative_dev(self):
        dev_sum = 0
        avr_decoding_time = self.average_decoding_time()
        for entry in self.entries:
            dev_sum += (entry["elapsed"]["decode"] - avr_decoding_time) ** 2
        return math.sqrt(dev_sum / len(self.entries)) / avr_decoding_time

    def average_decoding_time(self):
        return self.sum_decoding_time() / len(self.entries)


@dataclass
class TimeDistribution:
    lower: float = 1e-9
    # for QECP simulations, sometimes we need to capture very long time, so double the range
    upper: float = 1e9
    N: int = 4000
    counter: dict[int, int] = field(default_factory=lambda: {})
    underflow_count: int = 0
    overflow_count: int = 0

    @staticmethod
    def from_line(line: str) -> "TimeDistribution":
        # example: "<lower>1.000e-9<upper>1.000e0<N>2000[666]1[695]23[696]80[698]7[699]3[underflow]0[overflow]0"
        match = re.search(
            "<lower>([\+-e\d\.]+)<upper>([\+-e\d\.]+)<N>(\d+)((?:\[\d+\]\d+)*)\[underflow\](\d+)\[overflow\](\d+)",
            line,
        )
        lower = float(match.group(1))
        upper = float(match.group(2))
        N = int(match.group(3))
        counter = {}
        if match.group(4) != "":
            for ele in match.group(4)[1:].split("["):
                index, count = ele.split("]")
                counter[int(index)] = int(count)
        underflow_count = int(match.group(5))
        overflow_count = int(match.group(6))
        return TimeDistribution(
            lower=lower,
            upper=upper,
            N=N,
            counter=counter,
            underflow_count=underflow_count,
            overflow_count=overflow_count,
        )

    # the ratio between two latencies of neighboring bins
    @property
    def interval_ratio(self) -> float:
        return np.expm1(math.log(self.upper / self.lower) / self.N)

    def record(self, latency: float, count: int = 1):
        if latency < self.lower:
            self.underflow_count += count
        elif latency >= self.upper:
            self.overflow_count += count
        else:
            ratio = math.log(latency / self.lower) / math.log(self.upper / self.lower)
            index = math.floor(self.N * ratio)
            assert index < self.N
            if index in self.counter:
                self.counter[index] += count
            else:
                self.counter[index] = count

    def flatten(self) -> tuple[list[float], list[int]]:
        latencies = [
            self.lower * ((self.upper / self.lower) ** (i / self.N))
            for i in range(self.N)
        ]
        counters = [self.counter.get(i) or 0 for i in range(self.N)]
        counters[0] += self.underflow_count
        counters[1] += self.overflow_count
        return latencies, counters

    def to_line(self) -> str:
        line = f"<lower>{self.lower:.3e}<upper>{self.upper:.3e}<N>{self.N}"
        for index in sorted(self.counter.keys()):
            line += f"[{index}]{self.counter[index]}"
        line += f"[underflow]{self.underflow_count}[overflow]{self.overflow_count}"
        return line

    def assert_compatible_with(self, other: "TimeDistribution"):
        assert self.lower == other.lower
        assert self.upper == other.upper
        assert self.N == other.N

    def __add__(self, other: "TimeDistribution") -> "TimeDistribution":
        self.assert_compatible_with(other)
        result = TimeDistribution(**self.__dict__)
        result.underflow_count += other.underflow_count
        result.overflow_count += other.overflow_count
        for index in other.counter.keys():
            if index in result.counter:
                result.counter[index] += other.counter[index]
            else:
                result.counter[index] = other.counter[index]
        return result

    def latency_of(self, index: int) -> float:
        return self.lower * ((self.upper / self.lower) ** ((index + 0.5) / self.N))

    def count_records(self) -> int:
        return sum(self.counter.values())

    def average_latency(self) -> float:
        sum_latency = 0
        for index in self.counter.keys():
            sum_latency += self.counter[index] * self.latency_of(index)
        return sum_latency / self.count_records()

    def bias_latency(self, additional_latency: float) -> "TimeDistribution":
        distribution = TimeDistribution(lower=self.lower, upper=self.upper, N=self.N)
        for latency, count in zip(*self.flatten()):
            latency += additional_latency
            distribution.record(latency, count)
        return distribution

    def filter_latency_range(
        self, min_latency: float, max_latency: float, assert_count: int = 1
    ) -> "TimeDistribution":
        x_vec = []
        y_vec = []
        for latency, count in zip(*self.flatten()):
            if latency < min_latency or latency > max_latency:
                assert (
                    count <= assert_count
                ), f"[warning] latency {latency} has count {count} > {assert_count}"
                continue
            x_vec.append(latency)
            y_vec.append(count)
        distribution = TimeDistribution(
            lower=min(x_vec), upper=max(x_vec), N=len(x_vec)
        )
        for x, y in zip(x_vec, y_vec):
            distribution.record(x, y)
        return distribution

    # smooth the distribution by combing adjacent bins
    def combine_bins(self, combine_bin: int = 1) -> "TimeDistribution":
        x_vec, y_vec = self.flatten()
        cx_vec = []
        cy_vec = []
        if len(x_vec) % combine_bin != 0:
            # append 0 data
            padding = math.ceil(len(x_vec) / combine_bin) - len(x_vec)
            for i in range(padding):
                x = x_vec[-1] * (self.interval_ratio ** (1 + i))
                x_vec.append(x)
                y_vec.append(0)
        for idx in range(len(x_vec) // combine_bin):
            start = idx * combine_bin
            end = (idx + 1) * combine_bin
            x = sum(x_vec[start:end]) / combine_bin
            y = sum(y_vec[start:end])
            cx_vec.append(x)
            cy_vec.append(y)
        distribution = TimeDistribution(
            lower=min(cx_vec), upper=max(cx_vec), N=len(cx_vec)
        )
        for x, y in zip(cx_vec, cy_vec):
            distribution.record(x, y)
        return distribution

    def fit_exponential_tail(
        self, f_range: tuple[float, float] | None = None
    ) -> tuple[float, float]:
        counts_records = self.count_records()
        if f_range is None:
            f_range = (10 / counts_records, 1e5 / counts_records)
        min_f, max_f = f_range
        i_vec = []
        latencies, counters = self.flatten()
        # search from large latency to small latency
        for i, counter in reversed(list(enumerate(counters))):
            if counter / counts_records < min_f:
                continue
            if counter / counts_records >= max_f:
                break
            i_vec.append(i)
        fit_latency = np.array([latencies[i] for i in i_vec])
        fit_freq = np.array([counters[i] for i in i_vec]) / counts_records

        # assume freq / (latency * interval_ratio) = exp(A - B * latency)
        fit_y = np.log(fit_freq) - np.log(fit_latency) - np.log(self.interval_ratio)

        B, A = np.polyfit(-fit_latency, fit_y, 1)
        # print(f"P(L) = exp({A} - {B} * latency)")
        return A, B

    # find a latency where accumulated probability beyond this latency is higher than certain value
    def find_cut_off_latency(self, probability: float) -> float:
        cut_off_count = self.count_records() * probability
        assert cut_off_count >= 10, "otherwise not accurate enough"
        # accumulate from right most
        x_vec, y_vec = self.flatten()
        accumulated = 0
        for idx in reversed(range(0, len(x_vec))):
            accumulated += y_vec[idx]
            if accumulated >= cut_off_count:
                return x_vec[idx + 1]
