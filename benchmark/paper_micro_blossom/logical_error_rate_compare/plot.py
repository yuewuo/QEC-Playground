from dataclasses import dataclass
import matplotlib.pyplot as plt

d_vec = [3, 5, 7, 9, 11, 13, 15]


@dataclass
class DecoderResult:
    name: str
    data: any

    @staticmethod
    def with_name(name: str) -> "DecoderResult":
        data = []
        for d in d_vec:
            p_vec = []
            pL_vec = []
            err_vec = []
            with open(f"{name}_{d}.txt") as f:
                for line in f.readlines():
                    line = line.strip("\r\n ")
                    if line == "" or line.startswith("#"):
                        continue
                    lst = line.split(" ")
                    assert len(lst) >= 7
                    error_count = int(lst[4])
                    if error_count < 100:
                        continue

                    p_vec.append(float(lst[0]))
                    pL_vec.append(float(lst[5]))
                    err_vec.append(float(lst[5]) * float(lst[7]))
            data.append((p_vec, pL_vec, err_vec))
        return DecoderResult(name=name, data=data)

    def plot(self, compared: bool = False):
        for d, (p_vec, pL_vec, err_vec) in zip(d_vec, self.data):
            if not compared:
                plt.loglog(p_vec, pL_vec, "o-", label=f"d = {d}")
            else:
                plt.loglog(p_vec, pL_vec, ":")


results = [
    DecoderResult.with_name("unweighted_uf"),
    DecoderResult.with_name("weighted_uf"),
    DecoderResult.with_name("mwpm"),
]

if __name__ == "__main__":
    for result in results:
        plt.cla()
        result.plot()
        plt.xlim(8e-5, 1.2e-2)
        plt.ylim(1e-6, 1)
        plt.xlabel("physical error rate")
        plt.ylabel("logical error rate")
        plt.legend()
        plt.title(f"{result.name} accuracy")
        plt.savefig(f"{result.name}.pdf")

    # print comparison
    for result1, result2 in [
        (results[0], results[1]),
        (results[1], results[2]),
        (results[0], results[2]),
    ]:
        plt.cla()
        result2.plot()
        plt.gca().set_prop_cycle(None)
        result1.plot(compared=True)
        plt.xlim(8e-5, 1.2e-2)
        plt.ylim(1e-6, 1)
        plt.xlabel("physical error rate")
        plt.ylabel("logical error rate")
        plt.legend()
        plt.title(f"{result2.name} accuracy compared with {result1.name}")
        plt.savefig(f"{result2.name}_compare_with_{result1.name}.pdf")
