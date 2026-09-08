import radiate as rd

TARGET_NUM = 30

engine = rd.Engine.bit(TARGET_NUM).fitness(sum).limit(rd.Limit.score(TARGET_NUM))

print(engine.run())
