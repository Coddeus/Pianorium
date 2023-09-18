/// The x bounds of each piano key (both black and white, from left to right)
pub const LAYOUT: [[f32; 2]; 88] = [
    [-26. / 26., -25. / 26.],
    [-25.15 / 26., -24.55 / 26.],
    [-25. / 26., -24. / 26.],
    [-24. / 26., -23. / 26.],
    [-23.4 / 26., -22.8 / 26.],
    [-23. / 26., -22. / 26.],
    [-22.2 / 26., -21.6 / 26.],
    [-22. / 26., -21. / 26.],
    [-21. / 26., -20. / 26.],
    [-20.45 / 26., -19.85 / 26.],
    [-20. / 26., -19. / 26.],
    [-19.3 / 26., -18.7 / 26.],
    [-19. / 26., -18. / 26.],
    [-18.15 / 26., -17.55 / 26.],
    [-18. / 26., -17. / 26.],
    [-17. / 26., -16. / 26.],
    [-16.4 / 26., -15.8 / 26.],
    [-16. / 26., -15. / 26.],
    [-15.2 / 26., -14.6 / 26.],
    [-15. / 26., -14. / 26.],
    [-14. / 26., -13. / 26.],
    [-13.45 / 26., -12.85 / 26.],
    [-13. / 26., -12. / 26.],
    [-12.3 / 26., -11.7 / 26.],
    [-12. / 26., -11. / 26.],
    [-11.15 / 26., -10.55 / 26.],
    [-11. / 26., -10. / 26.],
    [-10. / 26., -9. / 26.],
    [-9.4 / 26., -8.8 / 26.],
    [-9. / 26., -8. / 26.],
    [-8.2 / 26., -7.6 / 26.],
    [-8. / 26., -7. / 26.],
    [-7. / 26., -6. / 26.],
    [-6.45 / 26., -5.85 / 26.],
    [-6. / 26., -5. / 26.],
    [-5.3 / 26., -4.7 / 26.],
    [-5. / 26., -4. / 26.],
    [-4.15 / 26., -3.55 / 26.],
    [-4. / 26., -3. / 26.],
    [-3. / 26., -2. / 26.],
    [-2.4 / 26., -1.8 / 26.],
    [-2. / 26., -1. / 26.],
    [-1.2 / 26., -0.6 / 26.],
    [-1. / 26., 0. / 26.],
    [0. / 26., 1. / 26.],
    [0.55 / 26., 1.15 / 26.],
    [1. / 26., 2. / 26.],
    [1.7 / 26., 2.3 / 26.],
    [2. / 26., 3. / 26.],
    [2.85 / 26., 3.45 / 26.],
    [3. / 26., 4. / 26.],
    [4. / 26., 5. / 26.],
    [4.6 / 26., 5.2 / 26.],
    [5. / 26., 6. / 26.],
    [5.8 / 26., 6.4 / 26.],
    [6. / 26., 7. / 26.],
    [7. / 26., 8. / 26.],
    [7.55 / 26., 8.15 / 26.],
    [8. / 26., 9. / 26.],
    [8.7 / 26., 9.3 / 26.],
    [9. / 26., 10. / 26.],
    [9.85 / 26., 10.45 / 26.],
    [10. / 26., 11. / 26.],
    [11. / 26., 12. / 26.],
    [11.6 / 26., 12.2 / 26.],
    [12. / 26., 13. / 26.],
    [12.8 / 26., 13.4 / 26.],
    [13. / 26., 14. / 26.],
    [14. / 26., 15. / 26.],
    [14.55 / 26., 15.15 / 26.],
    [15. / 26., 16. / 26.],
    [15.7 / 26., 16.3 / 26.],
    [16. / 26., 17. / 26.],
    [16.85 / 26., 17.45 / 26.],
    [17. / 26., 18. / 26.],
    [18. / 26., 19. / 26.],
    [18.6 / 26., 19.2 / 26.],
    [19. / 26., 20. / 26.],
    [19.8 / 26., 20.4 / 26.],
    [20. / 26., 21. / 26.],
    [21. / 26., 22. / 26.],
    [21.55 / 26., 22.15 / 26.],
    [22. / 26., 23. / 26.],
    [22.7 / 26., 23.3 / 26.],
    [23. / 26., 24. / 26.],
    [23.85 / 26., 24.45 / 26.],
    [24. / 26., 25. / 26.],
    [25. / 26., 26. / 26.],
]; // Look for LAYOUT[midinote-21]

/// The indices of the black notes in `LAYOUT`
pub const BLACK: [u8; 36] = [
    1, 4, 6, 9, 11, 13, 16, 18, 21, 23, 25, 28, 30, 33, 35, 37, 40, 42, 45, 47, 49, 52, 54, 57, 59, 61, 64, 66, 69, 71, 73, 76, 78, 81, 83, 85,
];
