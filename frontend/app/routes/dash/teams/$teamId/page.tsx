import { useEffect, useState } from "react"
import { useParams, Link, useNavigate } from "react-router"
import {
  Users,
  ArrowLeft,
  Loader2,
  Trash2,
  UserPlus,
  Shield,
  ShieldCheck,
  User,
  Mail,
} from "lucide-react"
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog"
import { api, type Team, type TeamMember } from "~/lib/api"

const roleLabels: Record<string, string> = {
  owner: "Owner",
  admin: "Admin",
  member: "Member",
}

const roleColors: Record<string, string> = {
  owner: "text-amber-600 dark:text-amber-400",
  admin: "text-blue-600 dark:text-blue-400",
  member: "text-zinc-600 dark:text-zinc-400",
}

export default function TeamDetailPage() {
  const { teamId } = useParams()
  const navigate = useNavigate()

  const [team, setTeam] = useState<Team | null>(null)
  const [members, setMembers] = useState<TeamMember[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [inviteOpen, setInviteOpen] = useState(false)
  const [inviteEmail, setInviteEmail] = useState("")
  const [inviteRole, setInviteRole] = useState("member")
  const [inviting, setInviting] = useState(false)

  const load = async () => {
    if (!teamId) return
    setLoading(true)
    setError(null)
    try {
      const [teamData, membersData] = await Promise.all([
        api.teams.get(teamId),
        api.teams.listMembers(teamId),
      ])
      setTeam(teamData)
      setMembers(membersData)
    } catch {
      setError("Failed to load team")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void load() }, [teamId])

  const handleInvite = async () => {
    if (!inviteEmail.trim() || !teamId) return
    setInviting(true)
    try {
      await api.teams.addMember(teamId, inviteEmail.trim(), inviteRole)
      setInviteOpen(false)
      setInviteEmail("")
      setInviteRole("member")
      await load()
    } catch {
      // error handled by api
    } finally {
      setInviting(false)
    }
  }

  const handleRoleChange = async (userId: string, role: string) => {
    if (!teamId) return
    try {
      await api.teams.updateMemberRole(teamId, userId, role)
      await load()
    } catch {
      // error handled by api
    }
  }

  const handleRemove = async (userId: string) => {
    if (!confirm("Remove this member from the team?")) return
    if (!teamId) return
    try {
      await api.teams.removeMember(teamId, userId)
      await load()
    } catch {
      // error handled by api
    }
  }

  const handleDeleteTeam = async () => {
    if (!teamId || !confirm("Delete this team permanently? This cannot be undone.")) return
    try {
      await api.teams.delete(teamId)
      navigate("/dash/teams")
    } catch {
      // error handled by api
    }
  }

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-zinc-500" />
      </div>
    )
  }

  if (error || !team) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-sm text-red-600">
          {error ?? "Team not found"}
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/dash/teams">
            <ArrowLeft className="h-4 w-4" />
          </Link>
        </Button>
        <div className="flex-1">
          <h1 className="text-3xl font-bold dark:text-white">{team.name}</h1>
          <p className="mt-1 text-zinc-600 dark:text-zinc-400">
            {team.slug} &middot; {team.member_count} member
            {team.member_count !== 1 ? "s" : ""}
          </p>
        </div>
        <div className="flex gap-2">
          <Dialog open={inviteOpen} onOpenChange={setInviteOpen}>
            <DialogTrigger asChild>
              <Button>
                <UserPlus className="mr-2 h-4 w-4" />
                Invite Member
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Invite Member</DialogTitle>
                <DialogDescription>
                  Add a team member by email address
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <label className="mb-1 block text-sm font-medium">Email</label>
                  <Input
                    placeholder="user@example.com"
                    value={inviteEmail}
                    onChange={(e) => setInviteEmail(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleInvite()}
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium">Role</label>
                  <Select value={inviteRole} onValueChange={setInviteRole}>
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="admin">Admin</SelectItem>
                      <SelectItem value="member">Member</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <DialogFooter>
                <Button
                  onClick={handleInvite}
                  disabled={inviting || !inviteEmail.trim()}
                >
                  {inviting && (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  Invite
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {team.role === "owner" && (
            <Button variant="destructive" onClick={handleDeleteTeam}>
              <Trash2 className="mr-2 h-4 w-4" />
              Delete
            </Button>
          )}
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Members
          </CardTitle>
          <CardDescription>
            {members.length} member{members.length !== 1 ? "s" : ""}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {members.length === 0 ? (
            <p className="py-4 text-center text-sm text-zinc-500">
              No members yet
            </p>
          ) : (
            <div className="divide-y">
              {members.map((member) => (
                <div
                  key={member.user_id}
                  className="flex items-center gap-4 py-3"
                >
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-zinc-100 dark:bg-zinc-800">
                    <User className="h-5 w-5 text-zinc-500" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium dark:text-white">
                      {member.first_name} {member.last_name}
                    </p>
                    <p className="flex items-center gap-1 text-xs text-zinc-500">
                      <Mail className="h-3 w-3" />
                      {member.email}
                    </p>
                  </div>

                  {member.role !== "owner" && team.role === "owner" ? (
                    <Select
                      value={member.role}
                      onValueChange={(role) =>
                        handleRoleChange(member.user_id, role)
                      }
                    >
                      <SelectTrigger className="w-28">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="admin">Admin</SelectItem>
                        <SelectItem value="member">Member</SelectItem>
                      </SelectContent>
                    </Select>
                  ) : (
                    <span
                      className={`flex items-center gap-1 text-sm font-medium ${roleColors[member.role] ?? ""}`}
                    >
                      {member.role === "owner" ? (
                        <ShieldCheck className="h-4 w-4" />
                      ) : member.role === "admin" ? (
                        <Shield className="h-4 w-4" />
                      ) : (
                        <User className="h-4 w-4" />
                      )}
                      {roleLabels[member.role] ?? member.role}
                    </span>
                  )}

                  {member.role !== "owner" &&
                    (team.role === "owner" || team.role === "admin") && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemove(member.user_id)}
                      >
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </Button>
                    )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
