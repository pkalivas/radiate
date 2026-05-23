import radiate as rd

metrics = rd.MetricSet(one=list(range(10)), two=list(range(10, 20)), three=4)
metrics.upsert("four", 2)
metrics.upsert("four", 3)
metrics.upsert("four", 4)
metrics.upsert("four", 5)


print(metrics.dashboard())

list_temp = rd.List(rd.Float32)

print(list_temp)
