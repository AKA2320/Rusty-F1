import fastf1
# fastf1.Cache.enable_cache('.cache') 
import json
import os


session = fastf1.get_session(2023,'Monaco','R')
session.load(telemetry=True)


drivers_list = session.drivers # LIST

results = session.results # DF
abv = results[['DriverNumber','Abbreviation']].to_dict()['Abbreviation']
abv = {v: k for k, v in abv.items()}

team_colors = results[['Abbreviation','TeamColor']].set_index('Abbreviation').to_dict()['TeamColor']
for i,j in team_colors.items():
    team_colors[i] = '#'+str(j)

positional_data = session.pos_data # DICT
positional_data_dict = {k: df[['X', 'Y']].to_dict() for k, df in positional_data.items()}

fastest_lap_pos = session.laps.pick_fastest().get_pos_data() # DF
fastest_lap_pos = fastest_lap_pos[['X', 'Y']].to_dict()

rotation = session.get_circuit_info().rotation # FLOAT

winner = results[(results['Position'] == 1.0) | (results['Position'] == 1)]['Abbreviation'].iloc[0]

# Saving
os.makedirs('data', exist_ok=True)
with open('data/drivers_list.json', 'w') as f:
    json.dump(drivers_list, f)

with open('data/winner.json', 'w') as f:
    json.dump(winner, f)

with open('data/abv.json', 'w') as f:
    json.dump(abv, f)

with open('data/team_colors.json', 'w') as f:
    json.dump(team_colors, f)

with open('data/positional_data_dict.json', 'w') as f:
    json.dump(positional_data_dict, f)

with open('data/rotation.json', 'w') as f:
    json.dump(rotation, f)

with open('data/fastest_lap_pos.json', 'w') as f:
    json.dump(fastest_lap_pos, f)