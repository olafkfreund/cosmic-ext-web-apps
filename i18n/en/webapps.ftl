app=Quick Web Apps
loading=Loading...
open=Open
number={ $number }
git-description = Git commit {$hash} on {$date}
delete=Delete
yes=Yes
no=No
confirm-delete=Are you sure you want to delete { $app }?
cancel=Cancel
downloader-canceled=Installing stopped.
help=Help
about=About
support-me=Support me
support-body=If you find this app useful, please consider to support author, by optional donation :)
settings=Settings
import-theme=Import theme
imported-themes=Imported themes
run-app=Run app
reset-settings=Reset settings
reset=Reset

# header
main-window={ $app }
view=View
create=Done
new-app=Create new
edit=Edit
close=Close
create-new-webapp=Create new Web App
icon-selector=Icon selector
icon-installer=Papirus Icons Installer

# common.rs
select-category=Select Category
select-browser=Select Browser

# home_screen.rs
installed-header=You have { $number ->
        [1] 1 web app
        *[other] { $number} web apps
    } installed:
not-installed-header=You don't have any web app installed. Please, press create button and create one.

# creator.rs
category=Category
web=Web
accessories=Accessories
education=Education
games=Games
graphics=Graphics
internet=Internet
office=Office
programming=Programming
sound-and-video=Sound & Video

browser=Browser

new-webapp-title=New Quick Web App
title=Title
url=URL
download-favicon=Download favicon
non-standard-arguments=Non-standard arguments
# keep navbar, isolated profile nad private mode small count of characters
navbar=Nav Bar
persistent-profile=Persistent Profile
private-mode=Private Mode
window-size=Window Size
decorations=Window Decorations
simulate-mobile=Try to simulate mobile device

# iconpicker.rs
icon-name-to-find=Icon name to find
my-icons=My icons
download=Download
search=Search
no-icons-found=No icons found. Try a different search term or upload a custom icon.

# icons_installator.rs
icons-installer-header=Please wait. Downloading icons...
icons-installer-message=This app requires icons to work with. In case we don't have access to your installed icons, we are installing Papirus icon pack to local directory so you can choose one icon for your web app from this pack.
icons-installer-finished-waiting=Downloading finished. Waiting 3 seconds to close this window..

# warning.rs
warning=You don't meet requirements
    .success=You can create new Web App
    .duplicate=  - Web App invalid. Maybe you already have this Web App?
warning-app-name = App name must be at least 3 characters
warning-app-url = Please enter a valid HTTP or HTTPS URL
    .wrong-icon =  - Selected icon is invalid. Select another one.
    .app-name=  - App name must be longer than 3 characters
    .app-url=  - You must provide valid URL starting with http:// or https://
    .app-icon=  - You must select an Icon for your launcher
    .app-browser=  - Please select a browser. Make sure at least one is installed system-wide or via Flatpak

# import/export
export-apps=Export all apps
import-apps=Import apps
toast-export-success=Apps exported successfully
toast-import-success=Apps imported successfully
toast-export-error=Failed to export apps
toast-import-error=Failed to import apps
duplicate=Duplicate

# file dialogs
file-dialog-export-title=Export Web Apps
file-dialog-import-title=Import Web Apps
file-dialog-open-theme=Open Theme
file-dialog-open-icons=Open multiple images
file-dialog-save=Save
file-dialog-import=Import
file-filter-ron=RON export
file-filter-ron-theme=Ron Theme
file-filter-png=PNG Image
file-filter-svg=SVG Images

# toast notifications
toast-app-saved=Web app saved successfully
toast-app-deleted=Web app deleted
toast-save-error=Failed to save web app

# custom CSS/JS injection
custom-css=Custom CSS
custom-css-placeholder=body {"{"} background: #1a1a2e; {"}"}
custom-js=Custom JavaScript
custom-js-placeholder=console.log('Hello from custom script');
custom-js-warning=Scripts run with full page access. Only use trusted code.

# user agent
user-agent=User Agent
user-agent-default=Default
user-agent-mobile=Mobile
user-agent-custom=Custom
user-agent-custom-label=Custom User Agent
user-agent-custom-placeholder=Mozilla/5.0 ...

# permissions
permission-camera=Allow Camera
permission-microphone=Allow Microphone
permission-geolocation=Allow Location
permission-notifications=Allow Notifications

# data management
clear-data=Clear Data
toast-data-cleared=Website data cleared successfully
toast-data-clear-error=Failed to clear website data
profile-data-size=Profile Data Size

# URL scheme handlers
url-schemes=URL Schemes
url-schemes-placeholder=mailto, webcal, slack

# editor sections (#49)
advanced-settings=Advanced Settings
basic-settings=Basic Settings

# quick actions (#52)
open-in-browser=Open in Browser

# running indicators (#50)
running-indicator=●

# grid view (#47)
toggle-view=Toggle View

# thumbnails (#48)
fetch-thumbnail=Load Preview

# privacy features (#53, #60, #61)
content-blocking=Block Ads & Trackers
block-third-party-cookies=Block Third-Party Cookies
block-webrtc=Block WebRTC IP Leak

# proxy (#54)
proxy-url=Proxy URL
proxy-url-placeholder=http://proxy.example.com:8080

# zoom & session (#55, #56)
zoom-level=Zoom Level
zoom-level-placeholder=1.0
restore-session=Restore Last Session

# usage statistics (#57)
launch-count=Launch Count
last-launched=Last Launched
never-launched=Never

# minimize to background (#59)
minimize-to-background=Minimize to Background on Close

# auto dark mode (#62)
auto-dark-mode=Auto Dark Mode

# bulk operations (#58)
bulk-select=Select
bulk-done=Done
bulk-delete=Delete Selected
bulk-export=Export Selected
toast-bulk-deleted=Selected apps deleted
toast-bulk-exported=Selected apps exported
