class HashMap:
    def __init__(self):
        self.buckets = [[]]
        self.total_items = 0

    def get(self, key):
        index = hash(key) % len(self.buckets)
        bucket = self.buckets[index]
        for item in bucket:
            if item[0] == key:
                return item[1]
    
    def get_items(self):
        result = []
        for bucket in self.buckets:
            result.extend(bucket)
        return result
    
    def rebalance(self):
        if 1.0 * self.total_items / len(self.buckets) < 0.75:
            return
        
        print('rebalancing')

        items = self.get_items()
        for i in range(len(self.buckets)):
            self.buckets[i].clear()
            self.buckets.append([])

        for item in items:
            index = hash(item[0]) % len(self.buckets)
            bucket = self.buckets[index]
            bucket.append(item)

    def set(self, key, value):
        index = hash(key) % len(self.buckets)
        bucket = self.buckets[index]

        for item in bucket:
            if item[0] == key:
                item[1] = value
                break
        else:
            bucket.append([key, value])
        
        self.total_items += 1
        self.rebalance()
    
    def pop(self, key):
        index = hash(key) % len(self.buckets)
        bucket = self.buckets[index]

        for i in range(len(bucket)):
            if bucket[i][0] == key:
                self.total_items -= 1
                return bucket.pop(i)[1]
