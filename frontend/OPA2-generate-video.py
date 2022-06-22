import os, sys, time
filepath = os.path.join(os.path.dirname(__file__), f"OPA2-MaximumWeightMatching.html")
fileurl = "file://" + filepath
video_folder = os.path.join(os.path.dirname(__file__), "OPA2-Video")
# print(fileurl)

# by default screen is not recorded, but just use existing screenshots
RECORD_SCREEN = False
if 'RECORD_SCREEN' in os.environ and os.environ["RECORD_SCREEN"] != "":
    RECORD_SCREEN = True

progress_min = -60
progress_max = 271
progress_scale = 0.01
wait_interval = 0.1  # sec

if RECORD_SCREEN:
    from selenium import webdriver
    from selenium.webdriver.chrome.service import Service
    options = webdriver.ChromeOptions()
    # options.add_argument('--kiosk')
    # options.add_argument("--start-fullscreen")
    driver = webdriver.Chrome(service=Service(os.path.join(video_folder, "chromedriver.exe")), options=options)

    driver.get(fileurl)
    driver.fullscreen_window()

    window_size = driver.get_window_size()
    print("window_size:", window_size)
    assert window_size["width"] == 3440 and window_size["height"] == 1440, "html page is designed (and only tested) under 3440 * 1440 screen"


    driver.find_element_by_tag_name('body').screenshot(os.path.join(video_folder, "starting.png"))

    for idx, progress in enumerate(range(progress_min, progress_max + 1)):
        driver.execute_script(f"app.progress = {progress_scale} * {progress}")
        time.sleep(wait_interval)
        driver.find_element_by_tag_name('body').screenshot(os.path.join(video_folder, f"{idx}.png"))

import cv2
height, width, layers = cv2.imread(os.path.join(video_folder, f"0.png")).shape
video = cv2.VideoWriter(os.path.join(video_folder, f"OPA2-MaximumWeightMatching.avi"), cv2.VideoWriter_fourcc(*'XVID'), 5.0, (width, height))
for idx, progress in enumerate(range(progress_min, progress_max + 1)):
    img = cv2.imread(os.path.join(video_folder, f"{idx}.png"))
    video.write(img)
video.release()
