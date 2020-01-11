import string

char_map = {s: i for s, i in zip(string.ascii_uppercase, range(1, 27))}
letter_map = {i: s for s, i in char_map.items()}


def to_u64(word):
    a = 0
    for c in word.upper().strip():
        a <<= 5
        a |= char_map[c]
    a = format(a, "064b")
    x = [int(a[8 * i:8 * i + 8], 2) for i in range(8)]
    return bytes(x)


def to_str(number):
    word = ""
    for i in range(12):
        val = 31 & number
        if val == 0:
            break
        number >>= 5
        word += letter_map[val]
    return word[::-1]


def split_prefixes(dictionary_path="./data/TWL06/TWL06Trimmed.txt", min_length=2, max_length=12):
    with open(dictionary_path, "r") as dictionary:
        lines = dictionary.readlines()
        for i in range(min_length, max_length + 1):
            with open(f"prefixes{i}L.txt", "w") as f:
                f.writelines((x[0:i] for x in lines if i <= len(x) <= max_length))


def prefixes_to_binary(prefix_path="./data/prefixes/", min_prefix=2, max_prefix=8):
    lines = []
    for i in range(min_prefix, max_prefix + 1):
        with open(f"{prefix_path}prefixes{i}L.txt") as f:
            lines.extend((to_u64(x) for x in f.readlines()))
    with open(f"{prefix_path}/binary.bin", "wb") as f:
        f.writelines(lines)


def file_to_binary(file_in, file_out, validate=True):
    lines = []
    with open(file_in) as f:
        lines.extend((to_u64(x) for x in f.readlines()))
    with open(file_out, "wb") as f:
        f.writelines(lines)

    if validate:
        check_correct(file_in, file_out)


def check_correct(file_in, file_out):
    with open(file_out, "rb") as binary_file, open(file_in, "r") as text_file:
        for word in text_file.readlines():
            str_repr = to_str(int.from_bytes(binary_file.read(8), byteorder='big'))
            if str_repr != word.strip():
                raise AssertionError(f"{str_repr} != {word}")


if __name__ == "__main__":
    prefixes_to_binary("../data/prefixes/", 2, 9)