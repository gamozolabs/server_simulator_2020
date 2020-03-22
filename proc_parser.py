from pprint import pprint
import re

formstring = """Processor {{
    manufacturer: "Intel",
    name: "{}",
    price: {:.2f},
    clock_rate: {},
    turbo_rate: None,
    avx512_rate: None,
    avx512_turbo_rate: None,
    cores: {},
    threads: {},
    avx512_fma_units: Some({}),
    typ: ProcessorType::{},
    scalability: {},
    mem_support: MemoryType::DDR4_{},
    mem_channels: {},
}},"""

pricere = re.compile("\$([0-9]+\.[0-9]{2})")
ghzre = re.compile("([0-9]+\.[0-9]{2}) GHz")
numberre = re.compile("([0-9]+)")

observed = set()

def process(fn):
    data = []
    for line in open(fn).read().splitlines():
        data.append(line.split(";"))

    column_header = data[0]
    data = data[1:]

    for data in data:
        data = dict(zip(column_header, data))
        
        price = float(pricere.findall(data["Recommended Customer Price"] + "$10000000.00")[0])

        if data["Sockets Supported"] == "":
            continue

        assert data["Name"] not in observed
        observed.update([data["Name"]])

        typ = ""
        pc = data["Product Collection"]
        if pc == "Intel® Xeon® Scalable Processors":
            typ = "XeonScalable"
        elif pc == "2nd Generation Intel® Xeon® Scalable Processors":
            typ = "XeonScalableV2"
        elif pc == "Intel® Xeon® W Processor":
            typ = "XeonW"
        elif pc == "Intel® Xeon® D Processor":
            typ = "XeonD"
        else:
            assert pc != pc, pc
        typ += "_" + data["Sockets Supported"]

        scale = int(numberre.findall(data["Scalability"])[0])
        assert scale in [1, 2, 4, 8]

        #print(data)
        max_mem_speed = max(map(int, numberre.findall((data["Maximum Memory Speed"] + data["Memory Types"]))))
        mem_chan = int(data["Max # of Memory Channels"])

        if max_mem_speed == 2666:
            max_mem_speed = 2667

        res = formstring.format(
            data["Name"],
            price,
            ghzre.findall(data["Processor Base Frequency"])[0],
            int(data["# of Cores"]),
            int(data["# of Threads"]),
            int(data["# of AVX-512 FMA Units"]),
            typ,
            scale,
            max_mem_speed,
            mem_chan,
        )
        print(res)

process("xeon_w.csv")
process("xeon_scalable_gen2.csv")

