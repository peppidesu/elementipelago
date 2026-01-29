import dotenv from "dotenv";
import fs from "node:fs/promises";

const config = dotenv.config().parsed;

const fetchUser = async (id) => {
    const response = await fetch(`https://discord.com/api/v9/users/${id}`, {
        headers: {
            Authorization: config.DISCORD_API_KEY,
        },
    });
    return await response.json();
};

const userIds = [
    "382561799742160896", // peppidesu
    "235482863250702336", // itepastra
    "110878826136907776", // hopop
    "1429874676590575907", // sarn kast
    "329656222213079053", // eeveon
    "1296648831370268756", // gleamingk111
];

Promise.all(
    userIds.map(async (id) => {
        const data = await fetchUser(id);
        return [id, { username: data.username, display: data.global_name }];
    }),
).then((data) => {
    fs.writeFile("./public/discord-users.json", JSON.stringify(Object.fromEntries(data)));
});
