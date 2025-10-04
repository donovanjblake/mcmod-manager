"""A mod manager for Minecraft written in Python that uses the Modrinth Labrinth API."""

import labrinth_session

if __name__ == "__main__":
    with labrinth_session.LabrinthSession() as session:
        print("Labrinth session initialized.")
        response = session.test_connection()
        print(f"connection test: {response}")
        response = session.get_project_version("iris", "1.21.5", "fabric")
        print(f"check iris: {response}")
