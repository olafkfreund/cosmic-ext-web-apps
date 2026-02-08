app=Rychlé webové aplikace
loading=Načítání...
open=Otevřít
number={ $number }
git-description = Git commit {$hash} dne {$date}
delete=Smazat
yes=Ano
no=Ne
confirm-delete=Opravdu chcete smazat { $app }?
cancel=Zrušit
downloader-canceled=Instalace zastavena.
help=Nápověda
about=O aplikaci
support-me=Podpořte mě
support-body=Pokud se vám tato aplikace líbí, zvažte prosím podporu autora dobrovolným příspěvkem. :)
settings=Nastavení
import-theme=Importovat motiv
imported-themes=Importované motivy
run-app=Spustit aplikaci
reset-settings=Obnovit nastavení
reset=Obnovit

# header
main-window={ $app }
view=Zobrazení
create=Hotovo
new-app=Vytvořit novou
edit=Úpravy
close=Zavřít
create-new-webapp=Vytvořit novou webovou aplikaci
icon-selector=Výběr ikony
icon-installer=Instalátor ikon Papirus

# common.rs
select-category=Vybrat kategorii
select-browser=Vybrat prohlížeč

# home_screen.rs
installed-header=Máte { $number ->
        [one] nainstalovanou
        [few] nainstalované
        *[other] nainstalovaných
    } { $number } { $number -> 
        [one] webovou aplikaci:
        [few] webové aplikace:
        *[other] webových aplikací:
    }
not-installed-header=Nemáte nainstalovanou žádnou webovou aplikaci. Stiskněte prosím tlačítko vytvořit.

# creator.rs
category=Kategorie
web=Web
accessories=Doplňky
education=Vzdělání
games=Hry
graphics=Grafika
internet=Internet
office=Kancelář
programming=Programování
sound-and-video=Zvuk a video

browser=Prohlížeč

new-webapp-title=Nová webová aplikace
title=Název
url=URL
download-favicon=Stáhnout favicon
non-standard-arguments=Nestandardní argumenty
# keep navbar, isolated profile nad private mode small count of characters
navbar=Navigační lišta
persistent-profile=Trvalý profil
private-mode=Soukromý režim
window-size=Velikost okna
decorations=Dekorace oken
simulate-mobile=Zkusit nasimulovat mobilní zařízení

# iconpicker.rs
icon-name-to-find=Název ikony pro vyhledání
my-icons=Moje ikony
download=Stáhnout
search=Vyhledat

# icons_installator.rs
icons-installer-header=Vyčkejte prosím. Stahuji ikony...
icons-installer-message=Tato aplikace vyžaduje ikony pro svou práci. Pokud nebude přístup k vámi nainstalovaným ikonám, nainstalují se ikony Papirus, ze kterých si můžete vybrat ikonu pro vaši webovou aplikaci.
icons-installer-finished-waiting=Stahování dokončeno. Během 3 sekund se toto okno zavře..

# warning.rs
warning=Nesplňujete požadavky
    .success=Můžete vytvořit novou webovou aplikaci
    .duplicate=  - Webová aplikace je neplatná. Možná už tuto webovou aplikaci máte?
    .wrong-icon =  - Vybraná ikona je neplatná. Vyberte některou jinou.
    .app-name=  - Název aplikace musí být delší než 3 znaky
    .app-url=  - Musíte poskytnou platnou URL začínající na http:// nebo https://
    .app-icon=  - Musíte vybrat ikonu pro váš spouštěč
    .app-browser=  - Vyberte prosím prohlížeč. Ujistěte se, že alespoň jeden je nainstalován v systému nebo skrze Flatpak
