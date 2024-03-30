import osmium
import sys

class NamesHandler(osmium.SimpleHandler):
    def node(self, n):
        if 'name' in n.tags:
            print(f'{n.location}: ' + n.tags['name'])

def main(osmfile):
    NamesHandler().apply_file(osmfile)
    return 0

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: python %s <osmfile>" % sys.argv[0])
        sys.exit(-1)

    exit(main(sys.argv[1]))