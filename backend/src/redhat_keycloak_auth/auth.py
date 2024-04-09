import seleniumwire.undetected_chromedriver as uc
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.common.by import By
import json
import time

data = None
with open('config.json') as config_file:
  data = json.load(config_file)

domain = data["mss_domain"] if "mss_domain" in data else "https://plan-editor-demo.satellite-soft.ru"

# opening chromedriver

chrome_options = uc.ChromeOptions()
# chrome_options.add_argument('--headless')
chrome_options.add_argument('--no-sandbox')
chrome_options.add_argument('--ignore-certificate-errors-spki-list')
chrome_options.add_argument('--ignore-ssl-errors')
driver = uc.Chrome(options=chrome_options, seleniumwire_options={})

# allow http

# driver.get("chrome://net-internals/#hsts")
# wait = WebDriverWait(driver, 30)
# domain_in = wait.until(EC.element_to_be_clickable((By.ID, 'domain-security-policy-view-delete-input')))
# domain_in = driver.find_element(By.ID, 'domain-security-policy-view-delete-input')
# domain_in.send_keys(domain)
# el = driver.find_element(By.ID, 'domain-security-policy-view-delete-submit')
# el.click()

# auth

driver.get(domain)

username = data["stnc_renaissance_username"]
password = data["stnc_renaissance_password"]

org_name = data["org_name"]
building_id = data["building_id"]
floor_id = data["floor_id"]

wait = WebDriverWait(driver, 30)
username_input = wait.until(EC.element_to_be_clickable((By.ID, 'username')))

username_input = driver.find_element(By.ID, 'username')
username_input.send_keys(username)
password_input = driver.find_element(By.ID, 'password')
password_input.send_keys(password)
login = driver.find_element(By.NAME, 'login')
login.click()

wait = WebDriverWait(driver, 30)
profile_btn = wait.until(EC.element_to_be_clickable((By.XPATH, "//*[starts-with(@class, 'Profile_profile_block_')]")))

profile_btn = driver.find_element(By.XPATH, "//*[starts-with(@class, 'Profile_profile_block_')]")
time.sleep(15)
profile_btn.click()

wait = WebDriverWait(driver, 30)
selector = wait.until(EC.element_to_be_clickable((By.XPATH, "//*[starts-with(@class, 'MuiInputBase-root')]")))
selector = driver.find_element(By.XPATH, "//*[starts-with(@class, 'MuiInputBase-root')]")
selector.click()

wait = WebDriverWait(driver, 30)
el = wait.until(EC.element_to_be_clickable((By.XPATH, f"//li[contains(text(), '{org_name}')]")))
el = driver.find_element(By.XPATH, f"//li[contains(text(), '{org_name}')]")
el.click()

# cookie search

time.sleep(5)
token = None
reqs = driver.requests
reqs.reverse()
for request in reqs:
  token = request.headers['Authorization']
  if token is not None:
    break

driver.quit()

with open('token.txt', 'w') as token_file:
  token_file.write(token)
