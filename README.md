# Myslice Calendar

Simple tool to export myslice class views into ical. It can then be imported into your favorite calendar apps like Google Calendar. This uses cookie for authentication.

## Usage

**First, obtain the required credentials**

You need to log into your myslice first. Then use devtool or plugins to obtain two required cookie for fetching the webpage:

- ITS-CSPRD101-80-PORTAL-PSJSESSIONID
- PS_TOKEN

Then export these env variable or create a .env file in the directory you will be invoking the program from:

- SESSION_ID: Value of "ITS-CSPRD101-80-PORTAL-PSJSESSIONID"
- TOKEN: Value of "PS_TOKEN"

**Then invoke the program**

You will see the result being stored in `./data` directory as `generated.ics`
