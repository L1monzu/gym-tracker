CREATE TABLE workout_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    position INTEGER NOT NULL
);

CREATE TABLE template_exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL REFERENCES workout_templates(id),
    exercise_name TEXT NOT NULL,
    position INTEGER NOT NULL,
    target_sets INTEGER NOT NULL DEFAULT 3,
    rep_range TEXT,
    rest_time TEXT,
    suggested_weight TEXT,
    notes TEXT
);

INSERT INTO workout_templates (name, position) VALUES
    ('Push', 1), ('Pull', 2), ('Upper', 3), ('Legs', 4);

INSERT INTO template_exercises (template_id, exercise_name, position, target_sets, rep_range, rest_time, suggested_weight, notes) VALUES
    (1, 'Barbell Bench Press', 1, 3, '6-8', '3 min', '~80-90 kg to start', 'Do this first while completely fresh, primary strength driver'),
    (1, 'Incline Dumbbell Press', 2, 3, '10-12', '90 sec', '22.5 kg', 'Upper chest and front delt tie-in'),
    (1, 'Chest Fly Machine', 3, 3, '12-15', '60 sec', '61 kg', 'Full stretch at the bottom, don''t rush the eccentric'),
    (1, 'Shoulder Press Machine', 4, 3, '8-10', '2 min', '60 kg', 'Moved before cables, compound before isolation'),
    (1, 'Cable Lateral Raises', 5, 3, '15-20', '60 sec', '5 kg', 'Slow eccentric, front delts already covered above'),
    (1, 'Overhead Cable Extension', 6, 3, '12-15', '60 sec', '35 kg', 'Elbows tucked, long head emphasis'),
    (1, 'Cable Kickbacks', 7, 2, '15', '45 sec', '7.5 kg', 'Finisher, 2 sets is plenty here'),

    (2, 'T-Bar Row', 1, 4, '6-8', '3 min', '45 kg', 'Promoted to lead exercise, heaviest compound, do it first'),
    (2, 'Lat Pulldown', 2, 3, '10-12', '2 min', '75 kg', 'Full hang at bottom, drive elbows to hips at top'),
    (2, 'Low Row (Seated Cable Row)', 3, 3, '10-12', '90 sec', '85 kg', 'Mid-back emphasis, squeeze hard at peak'),
    (2, 'Lat Pullover', 4, 3, '12-15', '60 sec', '25 kg', 'Great lat stretch, control the eccentric'),
    (2, 'Rear Delt Fly Machine', 5, 3, '15-20', '60 sec', '61 kg', 'Essential for shoulder health and upright boxing posture'),
    (2, 'Dumbbell Shrugs', 6, 3, '12-15', '60 sec', '32.5 kg', 'Moved from push day, traps belong with back work'),
    (2, 'Hammer Curl', 7, 3, '12-15', '60 sec', '10 kg', 'Forearm and brachialis, grip strength for punching'),
    (2, 'Preacher Curl Machine', 8, 3, '10-12', '60 sec', '46 kg', 'Strict isolation, no swinging'),

    (3, 'Barbell Overhead Press', 1, 4, '4-6', '3 min', '~50-60 kg to start', 'Strict, no leg drive, core braced throughout'),
    (3, 'Weighted Dips', 2, 3, '6-8', '2 min', 'Bodyweight + 10-20 kg', 'Slight forward lean = chest emphasis, upright = triceps'),
    (3, 'Incline Dumbbell Curl', 3, 3, '8-10', '90 sec', '~12-14 kg', 'Full stretch at bottom, no momentum'),
    (3, 'Skull Crushers', 4, 3, '8-10', '90 sec', '~30-35 kg barbell', 'Long head tricep, fires through every jab and cross'),
    (3, 'Face Pulls', 5, 3, '15-20', '60 sec', 'Light, form over weight', 'Shoulder health essential, do these every Saturday without exception'),

    (4, 'Hack Squat', 1, 3, '8-10', '2-3 min', '102.5 kg', 'Primary quad compound, do first while fully fresh'),
    (4, 'Leg Press', 2, 3, '12-15', '90 sec', '~120-140 kg', 'High foot placement for glutes and hamstrings'),
    (4, 'Seated Leg Curl', 3, 3, '12-15', '60 sec', '103 kg', 'Primary hamstring movement'),
    (4, 'Back Extension (45)', 4, 3, '12-15', '60 sec', '143 kg', 'Pause 1 sec at top, posterior chain and glutes'),
    (4, 'Leg Extension', 5, 2, '12-15', '60 sec', '126.5 kg', 'Quad finisher, 2 sets enough after hack squat'),
    (4, 'Calf Raises', 6, 3, '15-20', '45 sec', '210 kg', 'Slow eccentric, pause at top'),
    (4, 'Hip Abduction', 7, 2, '15-20', '45 sec', '150.5 kg', 'Superset with adduction to save time'),
    (4, 'Hip Adduction', 8, 2, '15-20', '45 sec', '150.5 kg', 'Superset with abduction, do back to back');