import { useEffect, useState } from "react"
import { Link } from "react-router"
import { Users, Plus, Settings, Loader2, Trash2, UserPlus } from "lucide-react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog"
import { api, type Team } from "~/lib/api"

export default function TeamsPage() {
  const [teams, setTeams] = useState<Team[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [createOpen, setCreateOpen] = useState(false)
  const [newName, setNewName] = useState("")
  const [creating, setCreating] = useState(false)

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await api.teams.list()
      setTeams(data)
    } catch {
      setError("Failed to load teams")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void load() }, [])

  const handleCreate = async () => {
    if (!newName.trim()) return
    setCreating(true)
    try {
      await api.teams.create(newName.trim())
      setCreateOpen(false)
      setNewName("")
      await load()
    } catch {
      // error handled by api
    } finally {
      setCreating(false)
    }
  }

  const handleDelete = async (teamId: string) => {
    if (!confirm("Delete this team? This cannot be undone.")) return
    try {
      await api.teams.delete(teamId)
      await load()
    } catch {
      // error handled by api
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold dark:text-white">Teams</h1>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Collaborate with team members on shared resources
          </p>
        </div>
        <Dialog open={createOpen} onOpenChange={setCreateOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              New Team
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create Team</DialogTitle>
              <DialogDescription>
                Give your team a name to get started
              </DialogDescription>
            </DialogHeader>
            <Input
              placeholder="Team name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            />
            <DialogFooter>
              <Button
                onClick={handleCreate}
                disabled={creating || !newName.trim()}
              >
                {creating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {loading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-zinc-500" />
        </div>
      ) : error ? (
        <Card>
          <CardContent className="py-8 text-center text-sm text-red-600">
            {error}
          </CardContent>
        </Card>
      ) : teams.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <Users className="mx-auto h-12 w-12 text-zinc-300 dark:text-zinc-600" />
            <p className="mt-4 text-lg font-medium dark:text-white">
              No teams yet
            </p>
            <p className="mt-1 text-sm text-zinc-500">
              Create a team to collaborate with others
            </p>
            <Button className="mt-4" onClick={() => setCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Create Team
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {teams.map((team) => (
            <Card key={team.id} className="group">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle>
                      <Link
                        to={`/dash/teams/${team.id}`}
                        className="hover:text-orange-600 dark:hover:text-orange-400"
                      >
                        {team.name}
                      </Link>
                    </CardTitle>
                    <CardDescription>{team.slug}</CardDescription>
                  </div>
                  <div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                    <Button
                      variant="ghost"
                      size="icon"
                      asChild
                    >
                      <Link to={`/dash/teams/${team.id}`}>
                        <Settings className="h-4 w-4" />
                      </Link>
                    </Button>
                    {team.role === "owner" && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleDelete(team.id)}
                      >
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </Button>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
                  <UserPlus className="h-4 w-4" />
                  {team.member_count} member{team.member_count !== 1 ? "s" : ""}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
